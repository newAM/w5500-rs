//! Binds socket 6, receives 180-byte datagrams and echoes 32 bytes back.
//!
//! Needs `host/sim.py --mode echo` running on the peer.
//!
//! # What this test cannot prove
//!
//! Byte order. This firmware decodes the inbound payload and encodes its reply;
//! if both use the wrong endianness the round trip is self-consistent and passes
//! anyway. Run `endian`, which checks against an externally-generated pattern,
//! before believing the payload is being read correctly.
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

/// Applies the static network configuration.
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
            error!("echo: network configuration failed: {error:?}");
            Timer::after_secs(5).await;
        }
    }

    // Destination written once here, never in the loop (optimization O3).
    if let Err(error) = w5500
        .udp_bind_to_peer(common::SOCKET, common::DEVICE_PORT, &common::PEER)
        .await
    {
        loop {
            error!("echo: bind failed: {error:?}");
            Timer::after_secs(5).await;
        }
    }
    info!(
        "echo: bound to port {}, peer {}",
        common::DEVICE_PORT,
        common::PEER
    );

    // Frame allocated once and reused (optimization O6).
    let mut frame: UdpFrame<{ common::FRAME_LEN }> = UdpFrame::new();
    let mut received_count: u32 = 0;
    let mut length_error_count: u32 = 0;
    let mut cycle_ticker = Ticker::every(Duration::from_micros(2000));
    let mut cycles_since_report: u32 = 0;

    loop {
        cycle_ticker.next().await;
        cycles_since_report += 1;

        match w5500.udp_recv_exact(common::SOCKET, &mut frame).await {
            Ok(()) => {
                received_count += 1;
                let sequence = common::payload::read_sequence(frame.payload()).unwrap_or(0);

                // Echo a fixed command set; `endian` is what checks the values.
                let commands: [f32; 8] = [1.0, 2.0, 0.5, -1.0, 0.0, 100.0, -0.25, 3.5];
                let reply = common::encode_reply(&commands);
                if let Err(error) = w5500.udp_send_exact(common::SOCKET, &reply).await {
                    error!("echo: send failed for sequence {sequence}: {error:?}");
                }
            }
            Err(Error::WouldBlock) => {}
            Err(Error::UnexpectedLength { expected, received }) => {
                length_error_count += 1;
                warn!("echo: datagram was {received} bytes, expected {expected}");
            }
            Err(other) => error!("echo: receive failed: {other:?}"),
        }

        // Every 2500 cycles at 2 ms is roughly every 5 seconds.
        if cycles_since_report >= 2500 {
            cycles_since_report = 0;
            info!("echo: {received_count} received, {length_error_count} wrong length");
        }
    }
}
