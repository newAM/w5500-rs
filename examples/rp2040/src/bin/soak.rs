//! Sustained 500 Hz receive plus 100 Hz send. **This is the acceptance test.**
//!
//! Needs `host/sim.py --mode soak --rate 500 --seconds 600` on the peer.
//! Run for at least 10 minutes.
//!
//! Verdict requires **both** sides to agree: this binary's own counters (zero
//! wrong-length datagrams, zero genuinely-skipped sequences) AND the host
//! script's independently-decoded sequence gaps at zero. Firmware that never
//! notices a dropped datagram also cannot report one, so the host's
//! independent count is what makes the zero meaningful. The host decodes its
//! gaps from the telemetry this binary's reply carries -- see "Reply format"
//! below.
//!
//! # Drain to latest
//!
//! Each cycle drains every buffered datagram and acts on the newest. On a
//! real-time system, falling behind and then processing stale input is
//! worse than skipping a cycle: the control loop would be reacting to the
//! past. Draining also self-corrects after a scheduling hiccup instead of
//! accumulating an ever-growing backlog.
//!
//! # `skipped_sequences` vs `drained_extra`
//!
//! Draining to latest means a cycle that catches up on a backlog legitimately
//! consumes several sequence numbers at once -- that is a design choice, not
//! loss. `drained_extra` counts exactly those deliberately-discarded
//! datagrams. `skipped_sequences` counts only sequence numbers that never
//! arrived at all: the gap between the previous and newest sequence number,
//! *after* subtracting however many were accounted for by this cycle's own
//! drain. The two counters are independent and mutually consistent: a busy
//! but lossless run can have `drained_extra > 0` and `skipped_sequences == 0`
//! at the same time.
//!
//! # Peer restarts
//!
//! Restarting `host/sim.py` mid-run resets its sequence counter to 0, which
//! looks like an enormous backward jump. That is treated as a peer restart --
//! logged, sequence tracking re-baselined -- rather than as billions of
//! skipped sequences. Genuine `u32` rollover (the sequence counter wrapping
//! after ~99 days at 500 Hz) is still handled correctly, via wrapping
//! arithmetic; see `RESTART_JUMP_THRESHOLD`.
//!
//! # Reply format
//!
//! Unlike `echo`/`endian`/`latency`, which reply with the production 8xf32
//! motor-command format, `soak`'s 32-byte reply is a diagnostic frame:
//!
//! - bytes 0..4:  last received sequence number (u32 little-endian)
//! - bytes 4..8:  this firmware's total received-datagram count (u32 little-endian)
//! - bytes 8..32: reserved, zero
//!
//! `soak`'s job is to carry telemetry for the host's independent cross-check,
//! not to exercise the motor-command encoding path -- that path stays covered
//! by `echo`, `endian` and `latency`, which still send real `f32` replies.
#![no_std]
#![no_main]

#[path = "../common.rs"]
#[allow(dead_code)]
mod common;

use embassy_executor::Spawner;
use embassy_time::{Duration, Instant, Ticker, Timer};
use log::{error, info, warn};
use panic_halt as _;
use w5500_hl::{
    fast_udp::{FastUdpAsync, UdpFrame},
    Error,
};
use w5500_ll::{aio::Registers, BufferSize, Sn};

/// Send every fifth cycle: 500 Hz in, 100 Hz out.
const SEND_EVERY_N_CYCLES: u32 = 5;

/// A backward sequence jump larger than this is treated as the peer having
/// restarted -- its sequence counter reset to a small number -- rather than
/// as a genuine gap or `u32` rollover. At 500 Hz sustained, wrapping the full
/// `u32` range legitimately takes about 99 days, so nothing this large is
/// reachable within one run unless the previous sequence number was already
/// near `u32::MAX`; see the rollover check next to its use below.
const RESTART_JUMP_THRESHOLD: u32 = 1_000_000;

const ALL_SOCKETS: [Sn; 8] = [
    Sn::Sn0,
    Sn::Sn1,
    Sn::Sn2,
    Sn::Sn3,
    Sn::Sn4,
    Sn::Sn5,
    Sn::Sn6,
    Sn::Sn7,
];

async fn configure_network(w5500: &mut common::W5500Device) -> Result<(), common::SpiError> {
    w5500.set_shar(&common::MAC).await?;
    w5500.set_sipr(&common::DEVICE_IP).await?;
    w5500.set_subr(&common::SUBNET).await?;
    w5500.set_gar(&common::GATEWAY).await?;

    // Zero every other socket's buffer allocation before growing socket 6's.
    // The W5500 has 16 KiB of RX buffer shared across all eight sockets, 2
    // KiB each by default; leaving the other seven at default while giving
    // socket 6 4 KiB would ask for 18 KiB total. The datasheet leaves
    // over-allocation undefined, and in practice the buffers overlap and
    // corrupt each other. Mirrors the `bufsize` binary.
    for socket in ALL_SOCKETS {
        if socket == common::SOCKET {
            continue;
        }
        w5500.set_sn_rxbuf_size(socket, BufferSize::KB0).await?;
        w5500.set_sn_txbuf_size(socket, BufferSize::KB0).await?;
    }

    // Socket 6 gets a modest 4 KiB: enough slack for a scheduling hiccup,
    // small enough that staleness stays bounded. See the `bufsize` binary.
    w5500
        .set_sn_rxbuf_size(common::SOCKET, BufferSize::KB4)
        .await?;
    w5500
        .set_sn_txbuf_size(common::SOCKET, BufferSize::KB2)
        .await?;
    Ok(())
}

/// Encodes `soak`'s 32-byte diagnostic reply. See the module doc "Reply
/// format" section for the layout and why it differs from the production
/// motor-command format the other diagnostic binaries send.
fn encode_diagnostic_reply(last_sequence: u32, received_total: u32) -> [u8; common::REPLY_LEN] {
    let mut reply: [u8; common::REPLY_LEN] = [0; common::REPLY_LEN];
    reply[0..4].copy_from_slice(&last_sequence.to_le_bytes());
    reply[4..8].copy_from_slice(&received_total.to_le_bytes());
    reply
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let (device, usb, _bus) = common::init_board();
    spawner.must_spawn(common::logger_task(usb));
    common::wait_for_host().await;

    let mut w5500 = common::W5500Device::new(device);

    if let Err(error) = configure_network(&mut w5500).await {
        loop {
            error!("soak: network configuration failed: {error:?}");
            Timer::after_secs(5).await;
        }
    }
    if let Err(error) = w5500
        .udp_bind_to_peer(common::SOCKET, common::DEVICE_PORT, &common::PEER)
        .await
    {
        loop {
            error!("soak: bind failed: {error:?}");
            Timer::after_secs(5).await;
        }
    }
    info!("soak: running at 500 Hz in, 100 Hz out -- leave this running >= 10 minutes");

    let mut frame: UdpFrame<{ common::FRAME_LEN }> = UdpFrame::new();

    let mut received_total: u32 = 0;
    let mut sent_total: u32 = 0;
    let mut empty_cycles: u32 = 0;
    let mut length_errors: u32 = 0;
    let mut other_errors: u32 = 0;
    let mut skipped_sequences: u32 = 0;
    let mut drained_extra: u32 = 0;
    let mut peer_restarts: u32 = 0;
    let mut worst_cycle_us: u64 = 0;
    let mut worst_cycle_since_report_us: u64 = 0;
    let mut last_sequence: Option<u32> = None;
    let mut cycle_index: u32 = 0;
    let mut cycles_since_report: u32 = 0;
    let started = Instant::now();

    let mut cycle_ticker = Ticker::every(Duration::from_micros(2000));

    loop {
        cycle_ticker.next().await;
        cycle_index += 1;
        cycles_since_report += 1;
        let cycle_started = Instant::now();

        // Drain to the newest datagram.
        let mut got_one = false;
        let mut newest_sequence: Option<u32> = None;
        let mut drained_extra_this_cycle: u32 = 0;
        loop {
            match w5500.udp_recv_exact(common::SOCKET, &mut frame).await {
                Ok(()) => {
                    if got_one {
                        drained_extra_this_cycle = drained_extra_this_cycle.saturating_add(1);
                    }
                    got_one = true;
                    received_total = received_total.saturating_add(1);
                    newest_sequence = common::payload::read_sequence(frame.payload());
                }
                Err(Error::WouldBlock) => break,
                Err(Error::UnexpectedLength { expected, received }) => {
                    length_errors = length_errors.saturating_add(1);
                    warn!("soak: datagram was {received} bytes, expected {expected}");
                }
                Err(other) => {
                    other_errors = other_errors.saturating_add(1);
                    error!("soak: receive failed: {other:?}");
                    break;
                }
            }
        }
        drained_extra = drained_extra.saturating_add(drained_extra_this_cycle);

        if !got_one {
            empty_cycles = empty_cycles.saturating_add(1);
        } else if let Some(sequence) = newest_sequence {
            if let Some(previous) = last_sequence {
                let expected_next = previous.wrapping_add(1);
                if sequence != expected_next {
                    let forward_gap = sequence.wrapping_sub(expected_next);
                    let previous_near_rollover = previous >= u32::MAX - RESTART_JUMP_THRESHOLD;
                    if forward_gap > RESTART_JUMP_THRESHOLD && !previous_near_rollover {
                        // Implausible as either a real gap or genuine u32
                        // rollover: the peer's sequence counter reset.
                        // Re-baseline instead of counting billions of
                        // skipped sequences.
                        peer_restarts = peer_restarts.saturating_add(1);
                        warn!(
                            "soak: sequence jumped from {previous} to {sequence} -- \
                             peer restarted, re-baselining"
                        );
                    } else {
                        // Datagrams genuinely lost in transit, excluding the
                        // ones this cycle's drain-to-latest deliberately
                        // discarded (see the module doc comment).
                        let genuinely_lost = forward_gap.saturating_sub(drained_extra_this_cycle);
                        skipped_sequences = skipped_sequences.saturating_add(genuinely_lost);
                    }
                }
            }
            last_sequence = Some(sequence);
        }

        if cycle_index.is_multiple_of(SEND_EVERY_N_CYCLES) {
            let reply = encode_diagnostic_reply(last_sequence.unwrap_or(0), received_total);
            match w5500.udp_send_exact(common::SOCKET, &reply).await {
                Ok(()) => sent_total = sent_total.saturating_add(1),
                Err(error) => {
                    other_errors = other_errors.saturating_add(1);
                    error!("soak: send failed: {error:?}");
                }
            }
        }

        let cycle_us = cycle_started.elapsed().as_micros();
        if cycle_us > worst_cycle_us {
            worst_cycle_us = cycle_us;
        }
        if cycle_us > worst_cycle_since_report_us {
            worst_cycle_since_report_us = cycle_us;
        }

        if cycles_since_report >= 2500 {
            cycles_since_report = 0;
            let elapsed_secs = started.elapsed().as_secs();
            info!(
                "soak: {elapsed_secs}s | rx {received_total}, tx {sent_total}, \
                 empty {empty_cycles}, skipped {skipped_sequences}, drained-extra {drained_extra}, \
                 peer-restarts {peer_restarts}"
            );
            info!(
                "  errors: {length_errors} wrong-length, {other_errors} other | \
                 worst cycle {worst_cycle_us} us (last 5 s: {worst_cycle_since_report_us} us)"
            );
            if length_errors == 0 && other_errors == 0 && skipped_sequences == 0 {
                info!("  ALL OK (firmware side -- confirm host/sim.py's decoded gaps agree)");
            } else {
                error!("  FAIL -- see counters above");
            }
            if worst_cycle_us >= 2000 {
                error!("  FAIL -- worst cycle exceeded the 2 ms budget");
            }
            worst_cycle_since_report_us = 0;
        }
    }
}
