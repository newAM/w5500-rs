//! Validates the payload byte order against an externally-generated pattern.
//!
//! Needs `host/sim.py --mode endian` running on the peer.
//!
//! # Why this is a separate binary
//!
//! `echo` cannot catch a byte-order bug. It decodes the inbound payload and
//! encodes its reply with the same code, so a firmware that is consistently
//! wrong produces a self-consistent round trip and passes. This is the same
//! structural blindness that let `mcp251xfd-rs` pass every loopback test at half
//! the intended CAN bit rate: both ends shared the same wrong assumption.
//!
//! The fix is an external reference. The host writes a known f32 pattern the
//! firmware never produced; the firmware compares it against constants compiled
//! into it. No consistently-wrong implementation can satisfy that.
//!
//! The boundary under test: **W5500 registers are big-endian** (datasheet
//! section 4.1 -- ports, lengths and pointers are all `from_be_bytes`), while
//! **the datagram payload is little-endian**. The driver must not confuse them.
#![no_std]
#![no_main]

// `common.rs` is shared across every diagnostic binary in this crate; each
// binary only exercises a subset of its constants and helpers, so an
// unqualified build would warn about the rest as dead code.
#[path = "../common.rs"]
#[allow(dead_code)]
mod common;

use embassy_executor::Spawner;
use embassy_time::{Duration, Ticker, Timer};
use log::{error, info, warn};
use panic_halt as _;
use w5500_hl::{
    Error,
    fast_udp::{FastUdpAsync, UdpFrame},
};
use w5500_ll::aio::Registers;

/// f32 equality within a tolerance, so a soft-float rounding difference is not
/// reported as an endianness fault. A byte-order error is never this close.
fn approximately_equal(left: f32, right: f32) -> bool {
    let difference = if left > right { left - right } else { right - left };
    difference < 0.0001
}

async fn configure_network(w5500: &mut common::W5500Device) -> Result<(), common::SpiError> {
    w5500.set_shar(&common::MAC).await?;
    w5500.set_sipr(&common::DEVICE_IP).await?;
    w5500.set_subr(&common::SUBNET).await?;
    w5500.set_gar(&common::GATEWAY).await?;
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
            error!("endian: network configuration failed: {error:?}");
            Timer::after_secs(5).await;
        }
    }
    if let Err(error) = w5500
        .udp_bind_to_peer(common::SOCKET, common::DEVICE_PORT, &common::PEER)
        .await
    {
        loop {
            error!("endian: bind failed: {error:?}");
            Timer::after_secs(5).await;
        }
    }
    info!("endian: waiting for a datagram from {}", common::PEER);

    let mut frame: UdpFrame<{ common::FRAME_LEN }> = UdpFrame::new();
    let mut checks_passed: u32 = 0;
    let mut checks_failed: u32 = 0;
    let mut cycle_ticker = Ticker::every(Duration::from_micros(2000));
    let mut cycles_since_report: u32 = 0;

    loop {
        cycle_ticker.next().await;
        cycles_since_report += 1;

        match w5500.udp_recv_exact(common::SOCKET, &mut frame).await {
            Ok(()) => {
                let payload = frame.payload();

                // Check 1: the origin address, decoded from the big-endian
                // W5500 receive header. A little-endian misread of port 49200
                // (0xC030) would give 12480 (0x30C0).
                let origin = frame.origin();
                let origin_ok = origin.port() == common::PEER.port();
                if !origin_ok {
                    error!(
                        "endian: origin port {}, expected {} -- W5500 header is big-endian",
                        origin.port(),
                        common::PEER.port()
                    );
                }

                // Check 2: the little-endian f32 pattern the host generated.
                let pattern_ok = match common::payload::read_pattern(payload) {
                    Some(pattern) => {
                        let mut all_match = true;
                        for (index, (actual, expected)) in pattern
                            .iter()
                            .zip(common::ENDIAN_PATTERN.iter())
                            .enumerate()
                        {
                            if !approximately_equal(*actual, *expected) {
                                error!(
                                    "endian: pattern[{index}] = {actual}, expected {expected}"
                                );
                                all_match = false;
                            }
                        }
                        all_match
                    }
                    None => {
                        error!("endian: payload too short to contain the pattern");
                        false
                    }
                };

                // Check 3: the filler byte, proving the payload is not shifted.
                let filler_ok = payload[common::payload::FILLER_OFFSET..]
                    .iter()
                    .all(|byte| *byte == common::payload::FILLER_BYTE);
                if !filler_ok {
                    error!("endian: filler bytes wrong -- payload is offset or truncated");
                }

                if origin_ok && pattern_ok && filler_ok {
                    checks_passed += 1;
                    if checks_passed <= 3 {
                        info!("endian: all checks OK (pass {checks_passed})");
                    }
                } else {
                    checks_failed += 1;
                    error!("endian: FAIL -- see above. This is a real byte-order bug.");
                }

                let reply = common::encode_reply(&common::ENDIAN_PATTERN);
                if let Err(error) = w5500.udp_send_exact(common::SOCKET, &reply).await {
                    error!("endian: send failed: {error:?}");
                }
            }
            Err(Error::WouldBlock) => {}
            Err(Error::UnexpectedLength { expected, received }) => {
                warn!("endian: datagram was {received} bytes, expected {expected}");
            }
            Err(other) => error!("endian: receive failed: {other:?}"),
        }

        // Every 2500 cycles at 2 ms is roughly every 5 seconds.
        if cycles_since_report >= 2500 {
            cycles_since_report = 0;
            if checks_failed == 0 && checks_passed > 0 {
                info!("endian: PASS -- {checks_passed} datagrams, 0 byte-order faults");
            } else if checks_passed == 0 {
                warn!("endian: no datagrams yet; is host/sim.py --mode endian running?");
            } else {
                error!(
                    "endian: FAIL -- {checks_failed} of {} datagrams bad",
                    checks_passed + checks_failed
                );
            }
        }
    }
}
