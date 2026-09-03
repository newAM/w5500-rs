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
use embassy_time::{with_timeout, Duration, Instant, Timer};
use log::{error, info, warn};
use panic_halt as _;
use w5500_ll::{aio::Registers, VERSION};

/// Rates to try, in Hz. `embassy-rp` rounds each down to what the divider can
/// reach, which is exactly what this binary exists to expose.
const CANDIDATE_RATES_HZ: [u32; 6] = [
    1_000_000, 8_000_000, 16_000_000, 31_250_000, 50_000_000, // rounds down to 31.25 MHz
    62_500_000,
];

/// Bytes moved per timing sample. Large enough that the fixed per-transaction
/// overhead does not dominate the measurement.
const TIMING_BYTES: usize = 1024;

/// Number of write/read-back rounds run at each candidate rate.
const INTEGRITY_ROUNDS: u32 = 24;

/// Ceiling on any single SPI operation in this binary.
///
/// A clock the board cannot sustain does not politely return an error: the
/// transfer stalls, the DMA future never resolves, and the executor stops
/// polling — which starves the USB logger and makes the serial port vanish, so
/// the operator sees the board "die" with no clue which rate did it. A
/// diagnostic must never be killed by the fault it is looking for, so every
/// SPI call here is bounded and a timeout is reported as a result, not a hang.
///
/// Generous by design: the slowest candidate moves 1024 bytes at ~1 MHz, which
/// is roughly 8 ms, so this cannot fire on a healthy bus.
const SPI_OPERATION_TIMEOUT: Duration = Duration::from_millis(500);

/// Reads a large block and returns the elapsed microseconds.
async fn time_block_read(w5500: &mut common::W5500Device) -> Result<Option<u64>, common::SpiError> {
    let mut scratch: [u8; TIMING_BYTES] = [0; TIMING_BYTES];
    let started = Instant::now();
    let outcome = with_timeout(
        SPI_OPERATION_TIMEOUT,
        w5500.sn_rx_buf(common::SOCKET, 0, &mut scratch),
    )
    .await;
    // Stop the clock BEFORE yielding: the yield below is executor bookkeeping,
    // not bus time, and at the fastest candidate it would be a larger error
    // than the measurement itself.
    let elapsed_us = started.elapsed().as_micros();
    // Same reason as `integrity_rounds`: give the USB task a turn.
    Timer::after_micros(200).await;
    match outcome {
        Ok(Ok(())) => Ok(Some(elapsed_us)),
        Ok(Err(error)) => Err(error),
        // Stalled: the bus cannot sustain this rate.
        Err(_) => Ok(None),
    }
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
        // Hand the executor back between rounds.
        //
        // This is not politeness, it is the difference between a working
        // binary and a board that never appears on USB at all. On a
        // single-threaded executor a task whose awaits keep completing
        // immediately is simply re-polled, and back-to-back SPI transfers
        // complete fast enough to do exactly that -- starving the USB task
        // that carries this binary's own log output. An earlier version of
        // this file hammered thousands of transfers with no timer await and
        // the board never finished USB enumeration, so it looked dead with no
        // diagnostic at all. `Timer` is used rather than a bare yield because
        // it guarantees a trip through the timer queue.
        Timer::after_micros(200).await;

        let mut written_frame: [u8; 188] = [0; 188];
        for (byte_index, byte) in written_frame.iter_mut().enumerate() {
            *byte = (byte_index as u8).wrapping_add(round_index as u8);
        }
        let mut read_back_frame: [u8; 188] = [0; 188];

        let round_trip_result = with_timeout(SPI_OPERATION_TIMEOUT, async {
            w5500
                .set_sn_tx_buf(common::SOCKET, 0, &written_frame)
                .await?;
            w5500
                .sn_tx_buf(common::SOCKET, 0, &mut read_back_frame)
                .await
        })
        .await;

        match round_trip_result {
            Ok(Ok(())) if read_back_frame == written_frame => {}
            // A stall counts as a failure: an unusable rate is not a clean one.
            Ok(Ok(())) | Ok(Err(_)) | Err(_) => failure_count += 1,
        }
    }
    failure_count
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let (device, usb, bus) = common::init_board();
    spawner.must_spawn(common::logger_task(usb));
    common::wait_for_host().await;

    // Deliberately slow to start. USB enumeration on the host can take several
    // seconds, and this binary is the one that deliberately drives the bus into
    // states it may not survive -- so the port must be up, and the operator
    // attached, before any of that begins. Otherwise a failure looks like a
    // dead board rather than a diagnosis.
    for remaining_seconds in (1..=8u32).rev() {
        info!("spiclock: starting sweep in {remaining_seconds} s (attach a terminal now)");
        Timer::after_secs(1).await;
    }

    let mut w5500 = common::W5500Device::new(device);

    loop {
        info!("--- spiclock: measuring achieved rate and integrity ceiling ---");

        for requested_hz in CANDIDATE_RATES_HZ {
            // Printed BEFORE the switch, and flushed, so that if this rate does
            // wedge the bus the last line on the wire names the culprit.
            info!("trying {requested_hz} Hz ...");
            Timer::after_millis(50).await;

            let mut spi_config = SpiConfig::default();
            spi_config.frequency = requested_hz;
            spi_config.phase = Phase::CaptureOnFirstTransition;
            spi_config.polarity = Polarity::IdleLow;
            bus.lock().await.set_config(&spi_config);

            // Confirm the chip still answers at all before trusting a timing.
            let version_result = with_timeout(SPI_OPERATION_TIMEOUT, w5500.version()).await;
            let version_result = match version_result {
                Ok(inner) => inner,
                Err(_) => {
                    error!(
                        "{requested_hz} Hz: VERSIONR read STALLED -- the bus cannot sustain this rate"
                    );
                    restore_default_clock(bus).await;
                    continue;
                }
            };
            match version_result {
                Ok(version) if version == VERSION => {}
                Ok(version) => {
                    warn!(
                        "{requested_hz} Hz: VERSIONR = 0x{version:02X} -- chip not responding correctly"
                    );
                    restore_default_clock(bus).await;
                    continue;
                }
                Err(error) => {
                    warn!("{requested_hz} Hz: VERSIONR read failed: {error:?}");
                    restore_default_clock(bus).await;
                    continue;
                }
            }

            match time_block_read(&mut w5500).await {
                Ok(None) => error!(
                    "{requested_hz} Hz: block read STALLED -- the bus cannot sustain this rate"
                ),
                Ok(Some(elapsed_us)) if elapsed_us > 0 => {
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
                Ok(Some(_)) => warn!("{requested_hz} Hz: timing too short to measure"),
                Err(error) => warn!("{requested_hz} Hz: block read failed: {error:?}"),
            }

            // Never carry a suspect clock into the next candidate's probe.
            restore_default_clock(bus).await;
        }

        info!("spiclock: use the highest rate with all rounds clean in common::SPI_FREQUENCY_HZ");
        info!("note: a request the divider cannot reach is rounded DOWN, so");
        info!("      'requested' and 'measured' differing is expected, not a fault");
        info!("note: 'measured' is end-to-end throughput, not the SCK rate. It");
        info!("      divides the bytes moved by the WHOLE call, so per-transfer");
        info!("      DMA setup and chip-select framing are amortised into it and");
        info!("      it always reads BELOW the true clock. Use it to compare rates");
        info!("      and find the ceiling, not as an absolute SCK measurement.");

        // Leave the bus at the crate default, so this binary is not
        // order-dependent with respect to the ones after it.
        restore_default_clock(bus).await;

        Timer::after_secs(5).await;
    }
}

/// Puts the bus back on [`common::SPI_FREQUENCY_HZ`].
///
/// Called after every candidate rate, not just at the end of a sweep: a rate
/// the board cannot sustain must not still be in force when the next probe
/// runs, or one bad rate would make every rate after it look broken.
async fn restore_default_clock(bus: &'static common::Bus) {
    let mut restore_config = SpiConfig::default();
    restore_config.frequency = common::SPI_FREQUENCY_HZ;
    restore_config.phase = Phase::CaptureOnFirstTransition;
    restore_config.polarity = Polarity::IdleLow;
    bus.lock().await.set_config(&restore_config);
}
