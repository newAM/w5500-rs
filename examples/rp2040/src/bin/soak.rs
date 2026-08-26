//! Sustained 500 Hz receive plus 100 Hz send. **This is the acceptance test.**
//!
//! Needs `host/sim.py --mode soak --rate 500 --seconds 600` on the peer.
//! Run for at least 10 minutes.
//!
//! Verdict requires **both** sides to agree: this binary's drop counter at zero
//! AND the host script reporting no sequence gaps. Firmware that never notices a
//! dropped datagram also cannot report one, so the host's independent count is
//! what makes the zero meaningful.
//!
//! # Drain to latest
//!
//! Each cycle drains every buffered datagram and acts on the newest. On a
//! real-time rig, falling behind and then processing stale simulator state is
//! worse than skipping a cycle: the control loop would be reacting to the past.
//! Draining also self-corrects after a scheduling hiccup instead of accumulating
//! an ever-growing backlog. Gaps in the sequence number are counted as skipped,
//! which is different from lost -- both are reported.
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
    Error,
    fast_udp::{FastUdpAsync, UdpFrame},
};
use w5500_ll::{BufferSize, aio::Registers};

/// Send every fifth cycle: 500 Hz in, 100 Hz out.
const SEND_EVERY_N_CYCLES: u32 = 5;

async fn configure_network(w5500: &mut common::W5500Device) -> Result<(), common::SpiError> {
    w5500.set_shar(&common::MAC).await?;
    w5500.set_sipr(&common::DEVICE_IP).await?;
    w5500.set_subr(&common::SUBNET).await?;
    w5500.set_gar(&common::GATEWAY).await?;
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
    let commands: [f32; 8] = [1.0, 2.0, 0.5, -1.0, 0.0, 100.0, -0.25, 3.5];

    let mut received_total: u32 = 0;
    let mut sent_total: u32 = 0;
    let mut empty_cycles: u32 = 0;
    let mut length_errors: u32 = 0;
    let mut other_errors: u32 = 0;
    let mut skipped_sequences: u32 = 0;
    let mut drained_extra: u32 = 0;
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
        loop {
            match w5500.udp_recv_exact(common::SOCKET, &mut frame).await {
                Ok(()) => {
                    if got_one {
                        drained_extra += 1;
                    }
                    got_one = true;
                    received_total += 1;
                    newest_sequence = common::payload::read_sequence(frame.payload());
                }
                Err(Error::WouldBlock) => break,
                Err(Error::UnexpectedLength { expected, received }) => {
                    length_errors += 1;
                    warn!("soak: datagram was {received} bytes, expected {expected}");
                }
                Err(other) => {
                    other_errors += 1;
                    error!("soak: receive failed: {other:?}");
                    break;
                }
            }
        }

        if !got_one {
            empty_cycles += 1;
        } else if let Some(sequence) = newest_sequence {
            if let Some(previous) = last_sequence {
                let expected_next = previous.wrapping_add(1);
                if sequence != expected_next {
                    skipped_sequences = skipped_sequences.saturating_add(sequence.wrapping_sub(expected_next));
                }
            }
            last_sequence = Some(sequence);
        }

        if cycle_index % SEND_EVERY_N_CYCLES == 0 {
            let reply = common::encode_reply(&commands);
            match w5500.udp_send_exact(common::SOCKET, &reply).await {
                Ok(()) => sent_total += 1,
                Err(error) => {
                    other_errors += 1;
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
                 empty {empty_cycles}, skipped {skipped_sequences}, drained-extra {drained_extra}"
            );
            info!(
                "  errors: {length_errors} wrong-length, {other_errors} other | \
                 worst cycle {worst_cycle_us} us (last 5 s: {worst_cycle_since_report_us} us)"
            );
            if length_errors == 0 && other_errors == 0 && skipped_sequences == 0 {
                info!("  ALL OK");
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
