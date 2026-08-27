//! Reports PHY link state and the configured network registers.
//!
//! Run after `spiclock`. This separates "no cable" from "no datagrams", which
//! are otherwise the same symptom in `echo`.
//!
//! `PHYCFGR` is W5500 datasheet section 4.1; the network registers `SHAR`,
//! `SIPR`, `SUBR` and `GAR` are section 4.1 as well.
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
use w5500_ll::{aio::Registers, LinkStatus};

/// Applies the static network configuration from `common`.
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

    loop {
        info!("--- link: PHY and network configuration ---");

        if let Err(error) = configure_network(&mut w5500).await {
            error!("network configuration failed: {error:?}");
            Timer::after_secs(5).await;
            continue;
        }

        match w5500.phycfgr().await {
            Ok(phy) => match phy.lnk() {
                LinkStatus::Up => info!("link: UP, speed {:?}, duplex {:?}", phy.spd(), phy.dpx()),
                LinkStatus::Down => {
                    warn!("link: DOWN");
                    warn!("  Check the cable, the magnetics, and PHY power.");
                    warn!("  Nothing downstream of here can work until this is UP.");
                }
            },
            Err(error) => error!("PHYCFGR read failed: {error:?}"),
        }

        // Read the configuration back from the chip rather than trusting the
        // write: a silently failing write looks identical to a working one.
        match w5500.sipr().await {
            Ok(address) if address == common::DEVICE_IP => info!("SIPR  = {address} (matches)"),
            Ok(address) => error!("SIPR  = {address}, expected {}", common::DEVICE_IP),
            Err(error) => error!("SIPR read failed: {error:?}"),
        }
        match w5500.subr().await {
            Ok(mask) if mask == common::SUBNET => info!("SUBR  = {mask} (matches)"),
            Ok(mask) => error!("SUBR  = {mask}, expected {}", common::SUBNET),
            Err(error) => error!("SUBR read failed: {error:?}"),
        }
        match w5500.gar().await {
            Ok(gateway) if gateway == common::GATEWAY => info!("GAR   = {gateway} (matches)"),
            Ok(gateway) => error!("GAR   = {gateway}, expected {}", common::GATEWAY),
            Err(error) => error!("GAR read failed: {error:?}"),
        }
        match w5500.shar().await {
            Ok(mac) if mac == common::MAC => info!("SHAR  = {mac} (matches)"),
            Ok(mac) => error!("SHAR  = {mac}, expected {}", common::MAC),
            Err(error) => error!("SHAR read failed: {error:?}"),
        }

        Timer::after_secs(5).await;
    }
}
