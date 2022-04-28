//! This uses a simulation of the W5500 registers that runs on your local machine.
//!
//! This does not require any embedded hardware to use.
//!
//! **Note:** This will communicate with local network services.

use mqttbytes::v5::{Connect, ConnectReturnCode, Packet};
use rand_core::OsRng;
use std::{
    thread::sleep,
    time::{Duration, Instant},
};
use w5500_regsim::W5500;
use w5500_tls::{
    hl::{
        io::{Read, Write},
        Hostname,
    },
    ll::{
        net::{Ipv4Addr, SocketAddrV4},
        Sn,
    },
    Client, Event, TlsWriter,
};

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
        .verbosity(4)
        .timestamp(stderrlog::Timestamp::Nanosecond)
        .init()
        .unwrap();

    let start: Instant = Instant::now();

    let mut w5500: W5500 = W5500::default();
    w5500.set_socket_buffer_logging(false);

    const SPORT: u16 = 11234;

    let mut rxbuf: [u8; 2048] = [0; 2048];
    const KEY: [u8; 32] = [
        0x2f, 0x42, 0xac, 0xe2, 0xb6, 0xbe, 0x16, 0x81, 0xb3, 0xd2, 0xfc, 0xdd, 0x4b, 0xb5, 0x7b,
        0x4f, 0xfe, 0x34, 0x84, 0xee, 0x77, 0xfd, 0xaa, 0x8e, 0x21, 0x6e, 0x32, 0x72, 0xcd, 0x78,
        0x25, 0x9d,
    ];
    let mut tls_client: Client<2048> =
        Client::new(TLS_SN, SPORT, HOSTNAME, HOST, b"test", &KEY, &mut rxbuf);

    loop {
        match tls_client.process(&mut w5500, &mut OsRng, monotonic_secs(start)) {
            Ok(Event::CallAfter(_)) => sleep(Duration::from_millis(50)),
            Ok(Event::ApplicationData) => {
                let mut buf = bytes::BytesMut::with_capacity(u16::MAX as usize);
                buf.resize(u16::MAX as usize, 0);

                let mut reader = tls_client.reader().unwrap();
                let len: u16 = reader.read(&mut buf).unwrap();
                reader.done().unwrap();

                match mqttbytes::v5::read(&mut buf, len.into()).unwrap() {
                    Packet::ConnAck(connack) => {
                        assert_eq!(connack.code, ConnectReturnCode::Success);
                    }
                    x => panic!("Unexpected response {x:?}"),
                }

                log::info!("MQTT connected");
                break;
            }
            Ok(Event::HandshakeFinished) => {
                let connect: Connect = Connect {
                    protocol: mqttbytes::Protocol::V5,
                    keep_alive: 60,
                    client_id: "w5500-test".to_string(),
                    clean_session: true,
                    last_will: None,
                    login: None,
                    properties: None,
                };
                let mut buf = bytes::BytesMut::new();
                connect.write(&mut buf).unwrap();

                let mut writer: TlsWriter<W5500> = tls_client.writer(&mut w5500).unwrap();
                writer.write_all(&buf).unwrap();
                writer.send().unwrap();
            }
            Ok(Event::Disconnect) => panic!("Unexpected disconnect"),
            Ok(Event::None) => sleep(Duration::from_millis(50)),
            Err(e) => panic!("Unexpected error: {e:?}"),
        }
    }
}
