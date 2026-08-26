#!/usr/bin/env python3
"""Host companion for the W5500 RP2040 diagnostics.

Stands in for the 6DOF flight simulator: sends 180-byte datagrams at a fixed
rate and receives the 32-byte replies. Standard library only.

Payload layout (all little-endian), matching src/common.rs:
    offset 0   u32  sequence number
    offset 4   u64  send timestamp, microseconds
    offset 12  8xf32 known pattern (checked by the `endian` binary)
    offset 44  filler 0xA5 to 180 bytes

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

# Must match common::ENDIAN_PATTERN exactly.
ENDIAN_PATTERN = (1.0, -2.0, 0.5, 3.25, -0.125, 100.0, 0.0, -7.5)


def build_payload(sequence_number, pattern):
    """Builds one 180-byte datagram."""
    payload = bytearray(b"\x00" * PAYLOAD_LEN)
    struct.pack_into("<I", payload, SEQUENCE_OFFSET, sequence_number & 0xFFFFFFFF)
    struct.pack_into("<Q", payload, TIMESTAMP_OFFSET, time.monotonic_ns() // 1000)
    struct.pack_into("<8f", payload, PATTERN_OFFSET, *pattern)
    for index in range(FILLER_OFFSET, PAYLOAD_LEN):
        payload[index] = FILLER_BYTE
    return bytes(payload)


def decode_reply(reply_bytes):
    """Decodes the 32-byte reply into 8 motor commands."""
    if len(reply_bytes) != REPLY_LEN:
        return None
    return struct.unpack("<8f", reply_bytes)


def run(arguments):
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

            # Try to send the datagram
            try:
                sock.sendto(payload, device_address)
            except (ConnectionResetError, OSError):
                timeout_count += 1
                if arguments.mode != "soak":
                    print(f"socket error sending to {device_address}")
            else:
                # Receive reply only if send succeeded
                try:
                    reply_bytes, _ = sock.recvfrom(4096)
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
                    if arguments.mode != "soak":
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

            if arguments.mode == "soak" and sent_count % (arguments.rate or 1) == 0:
                report(sent_count, received_count, timeout_count, round_trips_count, round_trips_sum, round_trips_max)
    except KeyboardInterrupt:
        print("\ninterrupted")

    print("\n--- final ---")
    report(sent_count, received_count, timeout_count, round_trips_count, round_trips_sum, round_trips_max)
    # The host's own view of loss, independent of the firmware's counters.
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

    return run(arguments)


if __name__ == "__main__":
    sys.exit(main())
