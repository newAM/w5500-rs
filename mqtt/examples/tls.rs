//! This uses a simulation of the W5500 registers that runs on your local machine.
//!
//! This does not require any embedded hardware to use.
//!
//! This will subscribe to all topics and loop continuously until stopped.
//!
//! **Note:** This will communicate with local network services.

use rand_core::OsRng;
use std::{
    str::from_utf8,
    thread::sleep,
    time::{Duration, Instant},
};
use w5500_mqtt::{
    hl::Hostname,
    ll::{
        net::{Ipv4Addr, SocketAddrV4},
        Sn,
    },
    tls::Client,
    Event,
};
use w5500_regsim::W5500;

// You will need to change these two values for your own network
const HOSTNAME: Hostname = Hostname::new_unwrapped("example-mqtt.local");
const HOST: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(10, 0, 0, 4), 8883);

const TLS_SN: Sn = Sn::Sn0;

fn monotonic_secs(start: Instant) -> u32 {
    Instant::now()
        .duration_since(start)
        .as_secs()
        .try_into()
        .unwrap()
}

fn main() {
    stderrlog::new()
        .verbosity(3)
        .timestamp(stderrlog::Timestamp::Nanosecond)
        .init()
        .unwrap();

    let start: Instant = Instant::now();

    let mut w5500: W5500 = W5500::default();
    w5500.set_socket_buffer_logging(false);

    const SPORT: u16 = 11234;

    let mut rxbuf: [u8; 1024] = [0; 1024];
    const KEY: [u8; 32] = [
        0x2f, 0x42, 0xac, 0xe2, 0xb6, 0xbe, 0x16, 0x81, 0xb3, 0xd2, 0xfc, 0xdd, 0x4b, 0xb5, 0x7b,
        0x4f, 0xfe, 0x34, 0x84, 0xee, 0x77, 0xfd, 0xaa, 0x8e, 0x21, 0x6e, 0x32, 0x72, 0xcd, 0x78,
        0x25, 0x9d,
    ];
    let mut client: Client<1024> =
        Client::new(TLS_SN, SPORT, HOSTNAME, HOST, b"test", &KEY, &mut rxbuf);

    loop {
        match client.process(&mut w5500, &mut OsRng, monotonic_secs(start)) {
            Ok(Event::CallAfter(_)) => (),
            Ok(Event::Publish(mut reader)) => {
                let mut payload_buf: [u8; 128] = [0; 128];
                let payload_len: u16 = reader
                    .read_payload(&mut payload_buf)
                    .expect("failed to read payload");
                let mut topic_buf: [u8; 128] = [0; 128];
                let topic_len: u16 = reader
                    .read_topic(&mut topic_buf)
                    .expect("failed to read payload");

                match (
                    from_utf8(&topic_buf[..topic_len.into()]),
                    from_utf8(&payload_buf[..payload_len.into()]),
                ) {
                    (Ok(topic), Ok(payload)) => log::info!("{topic} {payload}"),
                    _ => log::info!("payload and topic are not valid UTF-8"),
                }

                reader.done().unwrap();
            }
            // This does not handle failures
            Ok(Event::SubAck(ack)) => log::info!("{ack:?}"),
            // should never occur - we never unsubscribe
            Ok(Event::UnSubAck(ack)) => log::warn!("{ack:?}"),
            Ok(Event::ConnAck) => {
                client
                    .subscribe(&mut w5500, "#")
                    .expect("failed to send SUBSCRIBE");
            }
            Ok(Event::None) => sleep(Duration::from_millis(10)),
            Err(e) => panic!("Error occured: {e:?}"),
        }
    }
}
