//! Measures per-cycle receive and send cost in microseconds.
//!
//! Needs `host/sim.py --mode soak --rate 500` running on the peer.
//!
//! **This binary produces the number the acceptance criterion asks for.** The
//! design estimate is roughly 73 us at 62.5 MHz or 107 us at 31.25 MHz for a
//! receive-plus-send cycle, derived from byte counts plus a pessimistic 5 us per
//! SPI transaction. That is an estimate; this is the measurement.
//!
//! Worst case is reported both since boot and since the last report: a single
//! start-up outlier otherwise hides the steady-state figure for the whole run.
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
use w5500_ll::aio::Registers;

/// Coarse histogram edges in microseconds.
const HISTOGRAM_EDGES_US: [u64; 6] = [50, 100, 200, 500, 1000, 2000];

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
            error!("latency: network configuration failed: {error:?}");
            Timer::after_secs(5).await;
        }
    }
    if let Err(error) = w5500
        .udp_bind_to_peer(common::SOCKET, common::DEVICE_PORT, &common::PEER)
        .await
    {
        loop {
            error!("latency: bind failed: {error:?}");
            Timer::after_secs(5).await;
        }
    }
    info!("latency: measuring at {} Hz requested SPI clock", common::SPI_FREQUENCY_HZ);

    let mut frame: UdpFrame<{ common::FRAME_LEN }> = UdpFrame::new();
    let commands: [f32; 8] = [1.0, 2.0, 0.5, -1.0, 0.0, 100.0, -0.25, 3.5];

    let mut worst_receive_us: u64 = 0;
    let mut worst_send_us: u64 = 0;
    let mut worst_cycle_us: u64 = 0;
    let mut worst_cycle_since_report_us: u64 = 0;
    let mut histogram: [u32; 7] = [0; 7];
    let mut measured_cycles: u32 = 0;
    let mut cycles_since_report: u32 = 0;

    let mut cycle_ticker = Ticker::every(Duration::from_micros(2000));

    loop {
        cycle_ticker.next().await;
        cycles_since_report += 1;

        let cycle_started = Instant::now();
        let receive_started = Instant::now();
        let receive_result = w5500.udp_recv_exact(common::SOCKET, &mut frame).await;
        let receive_us = receive_started.elapsed().as_micros();

        match receive_result {
            Ok(()) => {
                let send_started = Instant::now();
                let reply = common::encode_reply(&commands);
                let send_result = w5500.udp_send_exact(common::SOCKET, &reply).await;
                let send_us = send_started.elapsed().as_micros();
                let cycle_us = cycle_started.elapsed().as_micros();

                if let Err(error) = send_result {
                    error!("latency: send failed: {error:?}");
                    continue;
                }

                measured_cycles += 1;
                if receive_us > worst_receive_us {
                    worst_receive_us = receive_us;
                }
                if send_us > worst_send_us {
                    worst_send_us = send_us;
                }
                if cycle_us > worst_cycle_us {
                    worst_cycle_us = cycle_us;
                }
                if cycle_us > worst_cycle_since_report_us {
                    worst_cycle_since_report_us = cycle_us;
                }

                let mut bucket = HISTOGRAM_EDGES_US.len();
                for (index, edge) in HISTOGRAM_EDGES_US.iter().enumerate() {
                    if cycle_us < *edge {
                        bucket = index;
                        break;
                    }
                }
                histogram[bucket] += 1;
            }
            Err(Error::WouldBlock) => {}
            Err(other) => error!("latency: receive failed: {other:?}"),
        }

        if cycles_since_report >= 2500 {
            cycles_since_report = 0;
            if measured_cycles == 0 {
                warn!("latency: no datagrams yet; is host/sim.py running?");
            } else {
                info!(
                    "latency: {measured_cycles} cycles | worst recv {worst_receive_us} us, \
                     send {worst_send_us} us, cycle {worst_cycle_us} us \
                     (last 5 s: {worst_cycle_since_report_us} us)"
                );
                info!(
                    "  <50us:{} <100:{} <200:{} <500:{} <1000:{} <2000:{} >=2000:{}",
                    histogram[0],
                    histogram[1],
                    histogram[2],
                    histogram[3],
                    histogram[4],
                    histogram[5],
                    histogram[6]
                );
                if worst_cycle_us >= 2000 {
                    error!("latency: FAIL -- worst cycle exceeded the 2 ms budget");
                }
            }
            worst_cycle_since_report_us = 0;
        }
    }
}
