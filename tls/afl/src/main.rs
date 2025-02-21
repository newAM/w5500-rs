mod shared;

use shared::{HOST, HOSTNAME, KEY, SPORT};
use w5500_fuzz::{FUZZ_SN, NotRng, W5500};
use w5500_tls::{Client, Error, Event};

fn main() {
    afl::fuzz!(|fuzz: &[u8]| {
        let mut buf: [u8; 2048] = [0; 2048];

        let mut client: Client<2048> =
            Client::new(FUZZ_SN, SPORT, HOSTNAME, HOST, b"test", &KEY, &mut buf);
        let mut w5500: W5500 = fuzz.into();

        loop {
            match client.process(&mut w5500, &mut NotRng::default(), 0) {
                Err(Error::NotConnected) => unreachable!(),
                Err(_) => break,
                Ok(Event::HandshakeFinished) => break,
                Ok(_) => (),
            }
        }
    })
}
