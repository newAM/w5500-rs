#![no_main]
use libfuzzer_sys::fuzz_target;
use w5500_fuzz::{FUZZ_SN, W5500};
use w5500_tls::{
    hl::Hostname,
    ll::net::{Ipv4Addr, SocketAddrV4},
    Client, Event,
};

const SRC_PORT: u16 = 0;
const HOSTNAME: Hostname = Hostname::new_unwrapped("localhost");
const SERVER: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 0);

fuzz_target!(|fuzz: &[u8]| {
    let mut buf: [u8; 2048] = [0; 2048];
    let mut client: Client<2048> = Client::new(
        FUZZ_SN,
        SRC_PORT,
        HOSTNAME,
        SERVER,
        b"test",
        &[0x55, 32],
        &mut buf,
    );
    let mut w5500: W5500 = fuzz.into();

    let mut mono: u32 = 0;
    loop {
        match client.process(&mut w5500, &mut rand_core::OsRng, mono) {
            Ok(Event::CallAfter(secs)) => mono += secs.saturating_sub(1),
            Ok(Event::HandshakeFinished) => break,
            Ok(_) => (),
            Err(_) => break,
        }
    }
});
