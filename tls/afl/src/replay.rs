mod shared;

use shared::{HOST, HOSTNAME, KEY, SPORT};
use std::{env, fs};
use w5500_fuzz::{FUZZ_SN, NotRng, W5500};
use w5500_tls::{Client, Error, Event};

fn main() {
    stderrlog::new()
        .verbosity(3)
        .timestamp(stderrlog::Timestamp::Nanosecond)
        .init()
        .unwrap();

    let args: Vec<String> = env::args().collect();
    let crashes: &String = args.get(1).expect("crashes directory was not provided");
    for entry in fs::read_dir(crashes).expect("Failed to read crash directory") {
        let entry = entry.unwrap();
        println!("Replaying crash: {:?}", entry.path());
        let data: Vec<u8> = fs::read(entry.path()).expect("failed to read crash file");
        let fuzz: &[u8] = &data;

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
    }
}
