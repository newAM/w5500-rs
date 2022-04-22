#![no_main]
use libfuzzer_sys::fuzz_target;
use w5500_fuzz::{FUZZ_SN, W5500};
use w5500_mqtt::{
    ll::net::{Ipv4Addr, SocketAddrV4},
    Client, Event,
};

const SRC_PORT: u16 = 0;
const SERVER: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 0);

fuzz_target!(|fuzz: &[u8]| {
    let mut client: Client = Client::new(FUZZ_SN, SRC_PORT, SERVER);
    let mut w5500: W5500 = fuzz.into();

    let mut mono: u32 = 0;
    loop {
        match client.process(&mut w5500, mono) {
            Ok(Event::None) => break,
            Ok(Event::Publish(mut reader)) => {
                let mut buf: [u8; 128] = [0; 128];
                let _ = reader.read_topic(&mut buf);
                let _ = reader.read_payload(&mut buf);
            }
            Ok(Event::SubAck { .. } | Event::UnsubAck { .. } | Event::ConnAck) => (),
            Ok(Event::CallAfter(secs)) => mono += secs.saturating_sub(1),
            Err(_) => break,
        }
    }
});
