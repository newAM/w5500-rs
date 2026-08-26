//! Measures the SPI clock the RP2040 actually produced, then sweeps upward to
//! find the highest rate this board stays clean at.
//!
//! Run this second, after `identify`.
//!
//! The RP2040 derives SPI frequency as `clk_peri / (CPSDVSR * (1 + SCR))` with
//! `CPSDVSR` even, so from a 125 MHz `clk_peri` the reachable rates near the top
//! are 62.5 MHz and 31.25 MHz with nothing between. A request for 50 MHz is
//! rounded **down** to 31.25 MHz without comment. This binary reports the
//! measured rate rather than the requested one, so that gap cannot silently
//! inflate every later timing measurement.
//!
//! This is the W5500 analogue of `mcp251xfd-rs`'s `bitrate`: an over-clocked SPI
//! bus corrupts reads intermittently rather than failing outright, and a
//! single-shot test cannot see it.
#![no_std]
#![no_main]

// `common.rs` is shared across every diagnostic binary in this crate; each
// binary only exercises a subset of its constants and helpers, so an
// unqualified build would warn about the rest as dead code.
#[path = "../common.rs"]
#[allow(dead_code)]
mod common;

use embassy_executor::Spawner;
use embassy_rp::spi::{Config as SpiConfig, Phase, Polarity};
use embassy_time::{Instant, Timer};
use log::{error, info, warn};
use panic_halt as _;
use w5500_ll::{VERSION, aio::Registers};

/// Rates to try, in Hz. `embassy-rp` rounds each down to what the divider can
/// reach, which is exactly what this binary exists to expose.
const CANDIDATE_RATES_HZ: [u32; 6] = [
    1_000_000,
    8_000_000,
    16_000_000,
    31_250_000,
    50_000_000, // rounds down to 31.25 MHz
    62_500_000,
];

/// Bytes moved per timing sample. Large enough that the fixed per-transaction
/// overhead does not dominate the measurement.
const TIMING_BYTES: usize = 2048;

/// Number of write/read-back rounds run at each candidate rate.
const INTEGRITY_ROUNDS: u32 = 64;

/// Reads a large block and returns the elapsed microseconds.
async fn time_block_read(w5500: &mut common::W5500Device) -> Result<u64, common::SpiError> {
    let mut scratch: [u8; TIMING_BYTES] = [0; TIMING_BYTES];
    let started = Instant::now();
    w5500.sn_rx_buf(common::SOCKET, 0, &mut scratch).await?;
    Ok(started.elapsed().as_micros())
}

/// Writes a pattern to the socket TX buffer and reads it back, `rounds` times.
///
/// Round trips the full 188-byte frame size the driver moves every 2 ms
/// cycle, so this exercises the same bulk DMA path an over-clocked bus
/// actually corrupts, rather than a small single-register access (which
/// `identify` already covers).
///
/// Returns the number of mismatching rounds. Intermittent corruption is the
/// signature of an over-clocked bus, so this repeats rather than checking once.
async fn integrity_rounds(w5500: &mut common::W5500Device, rounds: u32) -> u32 {
    let mut failure_count: u32 = 0;
    for round_index in 0..rounds {
        let mut written_frame: [u8; 188] = [0; 188];
        for (byte_index, byte) in written_frame.iter_mut().enumerate() {
            *byte = (byte_index as u8).wrapping_add(round_index as u8);
        }
        let mut read_back_frame: [u8; 188] = [0; 188];

        let round_trip_result = async {
            w5500.set_sn_tx_buf(common::SOCKET, 0, &written_frame).await?;
            w5500.sn_tx_buf(common::SOCKET, 0, &mut read_back_frame).await
        }
        .await;

        match round_trip_result {
            Ok(()) if read_back_frame == written_frame => {}
            Ok(()) => failure_count += 1,
            Err(_) => failure_count += 1,
        }
    }
    failure_count
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let (device, usb, bus) = common::init_board();
    spawner.must_spawn(common::logger_task(usb));
    common::wait_for_host().await;

    let mut w5500 = common::W5500Device::new(device);

    loop {
        info!("--- spiclock: measuring achieved rate and integrity ceiling ---");

        for requested_hz in CANDIDATE_RATES_HZ {
            let mut spi_config = SpiConfig::default();
            spi_config.frequency = requested_hz;
            spi_config.phase = Phase::CaptureOnFirstTransition;
            spi_config.polarity = Polarity::IdleLow;
            bus.lock().await.set_config(&spi_config);

            // Confirm the chip still answers at all before trusting a timing.
            match w5500.version().await {
                Ok(version) if version == VERSION => {}
                Ok(version) => {
                    warn!(
                        "{requested_hz} Hz: VERSIONR = 0x{version:02X} -- chip not responding correctly"
                    );
                    continue;
                }
                Err(error) => {
                    warn!("{requested_hz} Hz: VERSIONR read failed: {error:?}");
                    continue;
                }
            }

            match time_block_read(&mut w5500).await {
                Ok(elapsed_us) if elapsed_us > 0 => {
                    // Bits moved includes the 3-byte VDM header.
                    let bits_moved = ((TIMING_BYTES + 3) * 8) as u64;
                    let measured_hz = bits_moved.saturating_mul(1_000_000) / elapsed_us;
                    let failure_count = integrity_rounds(&mut w5500, INTEGRITY_ROUNDS).await;

                    if failure_count == 0 {
                        info!(
                            "{requested_hz} Hz requested -> ~{measured_hz} Hz measured, {INTEGRITY_ROUNDS}/{INTEGRITY_ROUNDS} rounds clean"
                        );
                    } else {
                        error!(
                            "{requested_hz} Hz requested -> ~{measured_hz} Hz measured, {failure_count}/{INTEGRITY_ROUNDS} rounds CORRUPT"
                        );
                    }
                }
                Ok(_) => warn!("{requested_hz} Hz: timing too short to measure"),
                Err(error) => warn!("{requested_hz} Hz: block read failed: {error:?}"),
            }
        }

        info!("spiclock: use the highest rate with all rounds clean in common::SPI_FREQUENCY_HZ");
        info!("note: a request the divider cannot reach is rounded DOWN, so");
        info!("      'requested' and 'measured' differing is expected, not a fault");

        // Leave the bus at the crate default before the next sweep, so this
        // binary is not order-dependent with respect to the ones after it.
        let mut restore_config = SpiConfig::default();
        restore_config.frequency = common::SPI_FREQUENCY_HZ;
        restore_config.phase = Phase::CaptureOnFirstTransition;
        restore_config.polarity = Polarity::IdleLow;
        bus.lock().await.set_config(&restore_config);

        Timer::after_secs(5).await;
    }
}
