//! Proves the SPI wiring to the W5500 -- the first thing to run on new
//! hardware.
//!
//! Reads `VERSIONR`, which the W5500 datasheet (section 4.1) fixes at 0x04, then
//! writes and reads back the source port register of the unused socket `Sn0`
//! to prove both data directions carry arbitrary bytes rather than a stuck
//! level. (`w5500_ll::aio::Registers` has no socket TX buffer read-back
//! accessor; `set_sn_port`/`sn_port` is a plain 16-bit read/write register
//! that serves the same purpose.)
//!
//! Output leaves over USB CDC-ACM serial: open the board's COM port in any
//! terminal. The sweep repeats every 5 s, so the report is still coming when you
//! connect.
#![no_std]
#![no_main]

// `common.rs` is shared across every diagnostic binary in this crate; each
// binary only exercises a subset of its constants and helpers, so an
// unqualified build would warn about the rest as dead code.
#[path = "../common.rs"]
#[allow(dead_code)]
mod common;

use embassy_executor::Spawner;
use embassy_time::Timer;
use log::{error, info, warn};
use panic_halt as _;
use w5500_ll::{aio::Registers, Sn, VERSION};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let (device, usb, _bus) = common::init_board();
    spawner.must_spawn(common::logger_task(usb));
    common::wait_for_host().await;

    let mut w5500 = common::W5500Device::new(device);

    loop {
        info!("--- identify: probing the W5500 on SPI0 ---");
        info!("requested SPI clock: {} Hz", common::SPI_FREQUENCY_HZ);

        match w5500.version().await {
            Ok(version) if version == VERSION => {
                info!("VERSIONR = 0x{version:02X}: OK");

                // Both directions carry arbitrary data, not a stuck level.
                // Sn0 is unused by this crate's binaries, so its source port
                // register is a safe, plain read/write 16-bit scratch value.
                const WRITTEN_PORT: u16 = 0xBEEF;
                let round_trip_result = async {
                    w5500.set_sn_port(Sn::Sn0, WRITTEN_PORT).await?;
                    w5500.sn_port(Sn::Sn0).await
                }
                .await;

                match round_trip_result {
                    Ok(read_back_port) if read_back_port == WRITTEN_PORT => {
                        info!("socket register round trip: OK");
                        info!("identify: PASS -- SPI wiring is good");
                    }
                    Ok(read_back_port) => {
                        error!(
                            "socket register round trip: wrote 0x{WRITTEN_PORT:04X}, read 0x{read_back_port:04X}"
                        );
                        error!("identify: FAIL -- SPI reads are unreliable, try `spiclock`");
                    }
                    Err(error) => error!("socket register access failed: {error:?}"),
                }
            }
            Ok(stuck @ (0x00 | 0xFF)) => {
                error!("VERSIONR = 0x{stuck:02X}: all-zeros or all-ones");
                error!("identify: FAIL -- MISO is not carrying data.");
                error!("  Check: MISO on GP20 and MOSI on GP19 (the RP2040 pin mux");
                error!("  allows no other assignment for SPI0), CS on GP17, and power.");
            }
            Ok(version) => {
                warn!("VERSIONR = 0x{version:02X}, expected 0x{VERSION:02X}");
                error!("identify: FAIL -- wrong or corrupt response; try a lower SPI clock");
            }
            Err(error) => error!("VERSIONR read failed: {error:?}"),
        }

        Timer::after_secs(5).await;
    }
}
