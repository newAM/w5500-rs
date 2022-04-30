mod shared;

use shared::{HOST, HOSTNAME, KEY, SPORT};
use std::{fs::File, thread::sleep, time::Duration};
use w5500_fuzz::{NotRng, FUZZ_SN};
use w5500_regsim::W5500;
use w5500_tls::{Client, Event};

fn main() {
    let mut w5500: W5500 = W5500::default();
    let corpus: File = File::create("corpus").unwrap();
    w5500.set_corpus_file(corpus);

    let mut rxbuf: [u8; 2048] = [0; 2048];

    let mut tls_client: Client<2048> =
        Client::new(FUZZ_SN, SPORT, HOSTNAME, HOST, b"test", &KEY, &mut rxbuf);

    loop {
        match tls_client.process(&mut w5500, &mut NotRng::default(), 0) {
            Ok(Event::CallAfter(_)) => sleep(Duration::from_millis(50)),
            Ok(Event::ApplicationData) => unreachable!(),
            Ok(Event::HandshakeFinished) => break,
            Ok(Event::Disconnect) => panic!("Unexpected disconnect"),
            Ok(Event::None) => sleep(Duration::from_millis(50)),
            Err(e) => panic!("Unexpected error: {e:?}"),
        }
    }
}
