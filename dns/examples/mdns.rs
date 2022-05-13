//! This is a simulation of the W5500 registers that runs on your local machine.
//!
//! This does not require any embedded hardware to use.
//!
//! **Note:** This will communicate with internal network services.

use std::time::{Duration, Instant};
use w5500_dns::{
    hl::{net::Eui48Addr, Error},
    ll::{Registers, Sn, VERSION},
    mdns::Client as MdnsClient,
    Hostname,
};
use w5500_regsim::W5500;

// DNS socket to use, this could be any of them
const DNS_SOCKET: Sn = Sn::Sn3;
const DNS_SRC_PORT: u16 = 45917;

const DEFAULT_MAC: Eui48Addr = Eui48Addr::new(0xDE, 0xAD, 0xBE, 0xEF, 0xFE, 0xED);

// change this to the name of a host on your network
const QUERY_HOSTNAME: Hostname = unsafe { Hostname::new_unchecked("_http._tcp.local") };

fn main() {
    // this enables the logging built into the register simulator
    stderrlog::new()
        .verbosity(3)
        .timestamp(stderrlog::Timestamp::Nanosecond)
        .init()
        .unwrap();

    let mut w5500: W5500 = W5500::default();
    assert_eq!(w5500.version().unwrap(), VERSION); // sanity check

    w5500.set_shar(&DEFAULT_MAC).unwrap();
    debug_assert_eq!(w5500.shar().unwrap(), DEFAULT_MAC);
    log::info!("DEFAULT_MAC = {DEFAULT_MAC}");

    let mut mdns_client: MdnsClient = MdnsClient::new(DNS_SOCKET, Some(DNS_SRC_PORT));

    let start: Instant = Instant::now();
    mdns_client
        .ptr_question(&mut w5500, &QUERY_HOSTNAME)
        .expect("failed to send MDNS query");

    loop {
        let mut buf: [u8; 1024] = [0; 1024];
        let mut response = loop {
            match mdns_client.response(&mut w5500, &mut buf) {
                Ok(x) => {
                    let elapsed: Duration = Instant::now().duration_since(start);
                    log::info!("DNS server responded in {elapsed:?}");
                    break x;
                }
                Err(Error::WouldBlock) => {}
                Err(x) => panic!("W5500 error: {x:?}"),
            }
        };

        while let Ok(Some(rr)) = response.next_rr() {
            println!("{rr:?}");
        }
        response.done().expect("done");
    }
}
