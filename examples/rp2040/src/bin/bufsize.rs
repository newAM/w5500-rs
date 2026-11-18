//! Assigns the socket RX/TX buffer allocation and confirms the chip accepted
//! it. This is optimization O7.
//!
//! The W5500 has 16 KiB of RX and 16 KiB of TX buffer, distributed across eight
//! sockets, 2 KiB each by default (datasheet section 4.2, `Sn_RXBUF_SIZE` and
//! `Sn_TXBUF_SIZE`). Only socket 6 is used here, so the others are set to zero
//! and socket 6 is given a larger share.
//!
//! **Not the maximum share, deliberately.** A 16 KiB receive buffer would hold
//! roughly 87 datagrams; on a real-time system a deep buffer is actively harmful,
//! because falling behind then means processing *stale* input rather
//! than dropping a cycle and re-syncing. 4 KiB is about 21 datagrams of slack:
//! enough to ride out a scheduling hiccup, small enough that staleness stays
//! bounded. The `soak` binary drains to the newest datagram for the same reason.
//!
//! The library hard-codes no distribution; this is entirely a consumer choice.
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
use log::{error, info};
use panic_halt as _;
use w5500_ll::{aio::Registers, BufferSize, Sn};

const RX_ALLOCATION: BufferSize = BufferSize::KB4;
const TX_ALLOCATION: BufferSize = BufferSize::KB2;

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

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let (device, usb, _bus) = common::init_board();
    spawner.must_spawn(common::logger_task(usb));
    common::wait_for_host().await;

    let mut w5500 = common::W5500Device::new(device);

    loop {
        info!("--- bufsize: socket buffer allocation ---");

        for socket in ALL_SOCKETS {
            let (rx_size, tx_size) = if socket == common::SOCKET {
                (RX_ALLOCATION, TX_ALLOCATION)
            } else {
                (BufferSize::KB0, BufferSize::KB0)
            };

            if let Err(error) = w5500.set_sn_rxbuf_size(socket, rx_size).await {
                error!("{socket:?}: set RX buffer size failed: {error:?}");
                continue;
            }
            if let Err(error) = w5500.set_sn_txbuf_size(socket, tx_size).await {
                error!("{socket:?}: set TX buffer size failed: {error:?}");
                continue;
            }

            // Read back: a rejected allocation must not look like a good one.
            let read_rx = w5500.sn_rxbuf_size(socket).await;
            let read_tx = w5500.sn_txbuf_size(socket).await;
            match (read_rx, read_tx) {
                (Ok(Ok(actual_rx)), Ok(Ok(actual_tx))) => {
                    if actual_rx == rx_size && actual_tx == tx_size {
                        info!("{socket:?}: RX {actual_rx:?}, TX {actual_tx:?} (accepted)");
                    } else {
                        error!(
                            "{socket:?}: asked RX {rx_size:?}/TX {tx_size:?}, chip reports RX {actual_rx:?}/TX {actual_tx:?}"
                        );
                    }
                }
                (rx_result, tx_result) => {
                    error!(
                        "{socket:?}: buffer size read back invalid: {rx_result:?} {tx_result:?}"
                    );
                }
            }
        }

        info!(
            "bufsize: socket {:?} has RX {RX_ALLOCATION:?}, TX {TX_ALLOCATION:?}",
            common::SOCKET
        );
        Timer::after_secs(5).await;
    }
}
