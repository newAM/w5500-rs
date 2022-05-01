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

    const DOCSRS: Hostname = Hostname::new_unwrapped("imac.local");

    let start: Instant = Instant::now();
    mdns_client
        .a_question(&mut w5500, &DOCSRS)
        .expect("failed to send MDNS query");

    loop {
        let mut buf: [u8; 63] = [0; 63];
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

        while let Some(ans) = response.next_answer().expect("W5500 error") {
            println!("{ans:?}");
        }
        response.done().expect("done");
    }
}
