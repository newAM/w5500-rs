//! This is a simulation of the W5500 registers that runs on your local machine.
//!
//! This does not require any embedded hardware to use.
//!
//! **Note:** This will communicate with external network services.

use rand_core::TryRng;
use std::io::Read as _;
use std::time::{Duration, Instant};
use w5500_dns::{
    Client, Hostname,
    hl::{Error, Udp},
    ll::{Registers, Sn, VERSION},
    servers as dns_servers,
};
use w5500_regsim::W5500;

// DNS socket to use, this could be any of them
const DNS_SOCKET: Sn = Sn::Sn3;

// this is ignored by the register simulation
const DNS_SOURCE_PORT: u16 = 45917;

#[derive(Debug)]
pub struct LinuxRng {
    file: std::fs::File,
}

impl LinuxRng {
    pub fn new() -> std::io::Result<Self> {
        let f = std::fs::File::open("/dev/urandom")?;
        Ok(LinuxRng { file: f })
    }
}

impl TryRng for LinuxRng {
    type Error = std::io::Error;

    fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
        let mut buf = [0u8; 4];
        self.try_fill_bytes(&mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }

    fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
        let mut buf = [0u8; 8];
        self.try_fill_bytes(&mut buf)?;
        Ok(u64::from_le_bytes(buf))
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Self::Error> {
        self.file.read_exact(dest)
    }
}

fn main() {
    // this enables the logging built into the register simulator
    stderrlog::new()
        .verbosity(3)
        .timestamp(stderrlog::Timestamp::Nanosecond)
        .init()
        .unwrap();

    let mut w5500: W5500 = W5500::default();
    assert_eq!(w5500.version().unwrap(), VERSION); // sanity check

    w5500
        .udp_bind(DNS_SOCKET, DNS_SOURCE_PORT)
        .expect("failed to bind");

    let random_number: u64 = LinuxRng::new()
        .expect("Failed to create LinuxRng")
        .try_next_u64()
        .expect("Failed to generate random number");

    let mut dns_client: Client = Client::new(
        Sn::Sn3,
        DNS_SOURCE_PORT,
        dns_servers::CLOUDFLARE,
        random_number,
    );

    const DOCSRS: Hostname = Hostname::new_unwrapped("docs.rs");

    let start: Instant = Instant::now();
    let id: u16 = dns_client
        .a_question(&mut w5500, &DOCSRS)
        .expect("failed to send DNS query");

    let mut buf: [u8; 63] = [0; 63];
    let mut response = loop {
        match dns_client.response(&mut w5500, &mut buf, id) {
            Ok(x) => {
                let elapsed: Duration = Instant::now().duration_since(start);
                log::info!("DNS server responded in {elapsed:?}");
                break x;
            }
            Err(Error::WouldBlock) => {
                let elapsed: Duration = Instant::now().duration_since(start);
                if elapsed > Duration::from_secs(3) {
                    panic!("Timeout: DNS server did not respond after {elapsed:?}")
                }
            }
            Err(x) => panic!("W5500 error: {x:?}"),
        }
    };

    while let Some(rr) = response.next_rr().unwrap() {
        println!("{rr:?}");
    }
    response.done().unwrap();
}
