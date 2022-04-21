#![no_main]
use libfuzzer_sys::fuzz_target;
use w5500_fuzz::{FUZZ_SN, W5500};
use w5500_sntp::{ll::net::Ipv4Addr, Client};

const SNTP_SRC_PORT: u16 = 0;
const SNTP_SERVER: Ipv4Addr = Ipv4Addr::new(0, 0, 0, 0);
const CLIENT: Client = Client::new(FUZZ_SN, SNTP_SRC_PORT, SNTP_SERVER);

fuzz_target!(|fuzz: &[u8]| {
    let mut w5500: W5500 = fuzz.into();
    CLIENT.on_recv_interrupt(&mut w5500);
});
