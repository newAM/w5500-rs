#!/usr/bin/env python3
"""Host companion for the W5500 RP2040 diagnostics.

Stands in for the 6DOF flight simulator: sends 180-byte datagrams at a fixed
rate and receives replies. Standard library only.

Outbound payload layout (all little-endian), matching src/common.rs:
    offset 0   u32  sequence number
    offset 4   u64  send timestamp, microseconds
    offset 12  8xf32 known pattern (checked by the `endian` binary)
    offset 44  filler 0xA5 to 180 bytes

Reply layout depends on mode:
  - `echo`/`endian`: the production format, 8xf32 motor commands.
  - `soak`: a diagnostic frame, NOT motor commands --
        offset 0   u32  firmware's last received sequence number
        offset 4   u32  firmware's total received-datagram count
        offset 8   reserved, zero, to 32 bytes
    `soak` sends only every fifth received datagram (100 Hz replies against
    500 Hz inbound), so its reply carries telemetry for this script's
    independent sequence-gap cross-check instead of a 1:1 echo. See
    `soak.rs`'s module doc for why.

Usage:
    python3 sim.py --mode echo
    python3 sim.py --mode endian
    python3 sim.py --mode soak --rate 500 --seconds 600
"""

import argparse
import socket
import struct
import sys
import time

PAYLOAD_LEN = 180
REPLY_LEN = 32
SEQUENCE_OFFSET = 0
TIMESTAMP_OFFSET = 4
PATTERN_OFFSET = 12
FILLER_OFFSET = 44
FILLER_BYTE = 0xA5
SEQUENCE_MODULUS = 1 << 32

# Must match common::ENDIAN_PATTERN exactly.
ENDIAN_PATTERN = (1.0, -2.0, 0.5, 3.25, -0.125, 100.0, 0.0, -7.5)


def build_payload(sequence_number, pattern):
    """Builds one 180-byte datagram."""
    payload = bytearray(b"\x00" * PAYLOAD_LEN)
    struct.pack_into("<I", payload, SEQUENCE_OFFSET, sequence_number & 0xFFFFFFFF)
    struct.pack_into("<Q", payload, TIMESTAMP_OFFSET, time.monotonic_ns() // 1000)
    struct.pack_into("<8f", payload, PATTERN_OFFSET, *pattern)
    for filler_index in range(FILLER_OFFSET, PAYLOAD_LEN):
        payload[filler_index] = FILLER_BYTE
    return bytes(payload)


def decode_reply(reply_bytes):
    """Decodes an echo/endian 32-byte reply into 8 motor commands."""
    if len(reply_bytes) != REPLY_LEN:
        return None
    return struct.unpack("<8f", reply_bytes)


def decode_diagnostic_reply(reply_bytes):
    """Decodes a `soak` 32-byte diagnostic reply.

    Returns (last_received_sequence, firmware_received_count), or None if the
    frame is the wrong length.
    """
    if len(reply_bytes) != REPLY_LEN:
        return None
    return struct.unpack_from("<II", reply_bytes, 0)


def run(arguments):
    """Lockstep send-then-receive loop for `echo` and `endian`.

    These modes reply 1:1, so blocking on a reply between sends is correct
    and simple. `soak` cannot use this loop -- see `run_soak`.
    """
    device_address = (arguments.device, arguments.port)
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    sock.bind((arguments.bind, arguments.bind_port))
    sock.settimeout(arguments.timeout)
    print(f"bound to {sock.getsockname()}, sending to {device_address}")

    pattern = ENDIAN_PATTERN
    interval = 1.0 / arguments.rate if arguments.rate > 0 else 0.0
    deadline = time.monotonic() + arguments.seconds if arguments.seconds else None

    sequence_number = 0
    sent_count = 0
    received_count = 0
    timeout_count = 0
    round_trips_count = 0
    round_trips_sum = 0.0
    round_trips_max = 0.0
    next_send = time.monotonic()

    try:
        while True:
            if deadline is not None and time.monotonic() >= deadline:
                break
            if arguments.count and sent_count >= arguments.count:
                break

            payload = build_payload(sequence_number, pattern)
            send_time = time.monotonic()

            try:
                sock.sendto(payload, device_address)
            except (ConnectionResetError, OSError):
                timeout_count += 1
                print(f"socket error sending to {device_address}")
            else:
                try:
                    reply_bytes, _source_address = sock.recvfrom(4096)
                    received_count += 1
                    rtt_us = (time.monotonic() - send_time) * 1e6
                    round_trips_count += 1
                    round_trips_sum += rtt_us
                    round_trips_max = max(round_trips_max, rtt_us)
                    commands = decode_reply(reply_bytes)
                    if commands is None:
                        print(f"reply was {len(reply_bytes)} bytes, expected {REPLY_LEN}")
                    elif arguments.mode == "echo" and received_count <= 3:
                        formatted = ", ".join(f"{value:.4f}" for value in commands)
                        print(f"reply {received_count}: [{formatted}]")
                except (socket.timeout, ConnectionResetError, OSError):
                    timeout_count += 1
                    print(f"timeout waiting for reply to sequence {sequence_number}")

            sent_count += 1
            sequence_number += 1

            if interval:
                next_send += interval
                sleep_for = next_send - time.monotonic()
                if sleep_for > 0:
                    time.sleep(sleep_for)
                else:
                    # Fell behind; resynchronize rather than accumulating drift.
                    next_send = time.monotonic()
    except KeyboardInterrupt:
        print("\ninterrupted")

    print("\n--- final ---")
    report(sent_count, received_count, timeout_count, round_trips_count, round_trips_sum, round_trips_max)
    lost = sent_count - received_count
    if lost:
        print(f"HOST-SIDE LOSS: {lost} of {sent_count} datagrams unanswered")
        return 1
    print("HOST-SIDE LOSS: none")
    return 0


def report(sent_count, received_count, timeout_count, round_trips_count, round_trips_sum, round_trips_max):
    if round_trips_count > 0:
        mean = round_trips_sum / round_trips_count
        print(
            f"sent {sent_count}, received {received_count}, timeouts {timeout_count}, "
            f"rtt mean {mean:.0f} us, worst {round_trips_max:.0f} us"
        )
    else:
        print(f"sent {sent_count}, received {received_count}, timeouts {timeout_count}")


def run_soak(arguments):
    """Decoupled send/receive loop for `soak`.

    `soak` replies only every fifth received datagram (100 Hz replies against
    a 500 Hz send rate), so blocking on a reply between every send -- as the
    lockstep `run` loop does -- times out four sends out of five and collapses
    the achieved send rate to roughly 1/s. Sends are driven strictly at
    `--rate`; each iteration then drains whatever replies are already sitting
    in the socket buffer, non-blocking, without slowing sends down.

    `sent - received` is meaningless here because of that 5:1 ratio, so it is
    not used as the loss metric. Instead, each diagnostic reply carries the
    firmware's last-received sequence number and its total received-datagram
    count (see the module docstring); comparing consecutive replies' spans of
    those two fields yields the number of sequence numbers that fell between
    two received datagrams and were never received at all -- a genuine,
    independently-decoded gap count. The firmware's own final received count
    is also reported alongside the host's sent count so the two can be
    compared directly; that comparison, together with the decoded gap count,
    is the real verdict.

    Uses only constant-memory running aggregates -- no per-datagram history.
    """
    device_address = (arguments.device, arguments.port)
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    sock.bind((arguments.bind, arguments.bind_port))
    sock.setblocking(False)
    print(f"bound to {sock.getsockname()}, sending to {device_address}")

    pattern = ENDIAN_PATTERN
    interval = 1.0 / arguments.rate if arguments.rate > 0 else 0.0
    deadline = time.monotonic() + arguments.seconds if arguments.seconds else None
    reports_per_send_batch = arguments.rate or 1

    sequence_number = 0
    sent_count = 0
    reply_count = 0
    malformed_reply_count = 0
    socket_error_count = 0

    device_last_sequence = None
    device_received_count = None
    host_decoded_gap_total = 0

    next_send = time.monotonic()
    next_report_at_sent_count = reports_per_send_batch

    try:
        while True:
            if deadline is not None and time.monotonic() >= deadline:
                break
            if arguments.count and sent_count >= arguments.count:
                break

            payload = build_payload(sequence_number, pattern)
            try:
                sock.sendto(payload, device_address)
                sent_count += 1
                sequence_number += 1
            except (ConnectionResetError, OSError):
                socket_error_count += 1

            # Drain every reply currently sitting in the socket buffer without
            # blocking -- do not wait for one, there may be none this cycle.
            while True:
                try:
                    reply_bytes, _source_address = sock.recvfrom(4096)
                except BlockingIOError:
                    break
                except (ConnectionResetError, OSError):
                    socket_error_count += 1
                    break

                reply_count += 1
                decoded = decode_diagnostic_reply(reply_bytes)
                if decoded is None:
                    malformed_reply_count += 1
                    print(f"reply was {len(reply_bytes)} bytes, expected {REPLY_LEN}")
                    continue

                reported_last_sequence, reported_received_count = decoded
                if device_last_sequence is not None and device_received_count is not None:
                    sequence_span = (reported_last_sequence - device_last_sequence) % SEQUENCE_MODULUS
                    received_span = (reported_received_count - device_received_count) % SEQUENCE_MODULUS
                    genuinely_lost = sequence_span - received_span
                    if genuinely_lost > 0:
                        host_decoded_gap_total += genuinely_lost
                device_last_sequence = reported_last_sequence
                device_received_count = reported_received_count

            if interval:
                next_send += interval
                sleep_for = next_send - time.monotonic()
                if sleep_for > 0:
                    time.sleep(sleep_for)
                else:
                    # Fell behind; resynchronize rather than accumulating drift.
                    next_send = time.monotonic()

            if sent_count >= next_report_at_sent_count:
                next_report_at_sent_count = sent_count + reports_per_send_batch
                report_soak(
                    sent_count, reply_count, socket_error_count,
                    device_last_sequence, device_received_count, host_decoded_gap_total,
                )
    except KeyboardInterrupt:
        print("\ninterrupted")

    print("\n--- final ---")
    report_soak(
        sent_count, reply_count, socket_error_count,
        device_last_sequence, device_received_count, host_decoded_gap_total,
    )
    print(f"HOST-DECODED SEQUENCE GAPS: {host_decoded_gap_total}")
    if device_received_count is not None:
        difference = sent_count - device_received_count
        print(
            f"host sent {sent_count}, firmware received {device_received_count} "
            f"(difference {difference}) -- compare against the decoded gap count above"
        )
    else:
        print("no diagnostic replies were ever decoded -- is soak.rs running and reachable?")

    if host_decoded_gap_total or malformed_reply_count:
        return 1
    return 0


def report_soak(sent_count, reply_count, socket_error_count, device_last_sequence, device_received_count, host_decoded_gap_total):
    if device_last_sequence is None:
        print(f"sent {sent_count}, replies {reply_count}, socket errors {socket_error_count}, no replies decoded yet")
        return
    print(
        f"sent {sent_count}, replies {reply_count}, socket errors {socket_error_count}, "
        f"device last-seq {device_last_sequence}, device rx-count {device_received_count}, "
        f"host-decoded gaps {host_decoded_gap_total}"
    )


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--mode", choices=["echo", "endian", "soak"], default="echo")
    parser.add_argument("--device", default="192.168.0.10")
    parser.add_argument("--port", type=int, default=49200)
    parser.add_argument("--bind", default="192.168.0.1")
    parser.add_argument("--bind-port", type=int, default=49200)
    parser.add_argument("--rate", type=int, default=500)
    parser.add_argument("--count", type=int, default=0)
    parser.add_argument("--seconds", type=float, default=0.0)
    parser.add_argument("--timeout", type=float, default=1.0)
    arguments = parser.parse_args()

    if arguments.mode == "echo":
        arguments.rate = arguments.rate or 10
        arguments.count = arguments.count or 10
    elif arguments.mode == "endian":
        arguments.rate = 10
        arguments.count = arguments.count or 20

    if arguments.mode == "soak":
        return run_soak(arguments)
    return run(arguments)


if __name__ == "__main__":
    sys.exit(main())
