//! This is a simulation of the W5500 registers that runs on your local machine.
//!
//! This does not require any embedded hardware to use.
//!
//! This code is very minimal to make this example readable.
//! Do as I say, not as I do: Hard coded DNS is bad.
//!
//! **Note:** This will communicate external network services.

use std::{
    thread::sleep,
    time::{Duration, Instant},
};

use w5500_hl::Udp;
use w5500_ll::{
    net::{Ipv4Addr, SocketAddrV4},
    Registers, Sn, VERSION,
};
use w5500_regsim::W5500;

// DNS socket to use, this could be any of them
const DNS_SOCKET: Sn = Sn::Sn3;

// this is ignored by the register simulation
const DNS_SOURCE_PORT: u16 = 1234;

// Cloudflare DNS
const DNS_SERVER: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(1, 1, 1, 1), 53);

// This is a hard coded DNS query for docs.rs
#[rustfmt::skip]
const QUERY: [u8; 25] = [
    // ID
    0xA7, 0x20,
    // non-recursive query
    0x01, 0x00,
    // one question
    0x00, 0x01,
    // 0 answer RRs
    0x00, 0x00,
    // 0 authority RRs
    0x00, 0x00,
    // 0 additonal RRs
    0x00, 0x00,
    // 4 byte label
    0x04,
    b'd', b'o', b'c', b's',
    // 2 byte label
    0x02,
    b'r', b's',
    // null terminator
    0x00,
    // QTYPE: A record query
    0x00, 0x01,
    // QCLASS: internet address
    0x00, 0x01,
];

fn main() {
    // this enables the logging built into the register simulator
    stderrlog::new()
        .verbosity(4)
        .timestamp(stderrlog::Timestamp::Nanosecond)
        .init()
        .unwrap();

    let mut w5500: W5500 = W5500::default();
    assert_eq!(w5500.version().unwrap(), VERSION); // sanity check

    // in a real embedded system there is a lot more boilerplate such as:
    // - DHCP (or setting a static IP)
    // - setting a valid EUI-48 MAC address
    // - Checking link up at the physical layer
    //
    // the register simulation allows us to cheat a little since your PC
    // (hopefully) already has a valid IP/MAC/Gateway/subnet mask

    w5500
        .udp_bind(DNS_SOCKET, DNS_SOURCE_PORT)
        .expect("Failed to bind the socket as UDP");

    let tx_bytes = w5500.udp_send_to(DNS_SOCKET, &QUERY, &DNS_SERVER).unwrap();
    assert_eq!(tx_bytes, QUERY.len());

    // in an embedded system you should wait for a socket interrupt
    // or at the very least have a timeout
    let start = Instant::now();
    let mut buf: [u8; 100] = [0; 100];
    let (rx_bytes, origin) = loop {
        match w5500.udp_recv_from(DNS_SOCKET, &mut buf) {
            Ok((num_bytes, origin)) => break (num_bytes, origin),
            Err(nb::Error::WouldBlock) => {
                sleep(Duration::from_millis(100));
                if Instant::now() - start > Duration::from_secs(3) {
                    panic!("Timeout waiting for udp_recv_from");
                }
            }
            Err(nb::Error::Other(e)) => panic!("Bus error: {}", e),
        }
    };

    assert_eq!(origin.ip(), DNS_SERVER.ip());
    let filled_buf = &buf[..rx_bytes];
    let mut buf_iter = filled_buf.iter();

    let mut next = || *buf_iter.next().expect("Buf is shorter than we expected");

    // ID should be the same
    assert_eq!(next(), QUERY[0]);
    assert_eq!(next(), QUERY[1]);

    // message type should be a response
    assert_eq!(next() & 0x80, 0x80);
    // byte 3 contains flags we do not care about for this example
    next();

    let questions = u16::from_be_bytes([next(), next()]);
    assert_eq!(questions, 1);

    let answer_rrs = u16::from_be_bytes([next(), next()]);
    println!("Answer RRs: {}", answer_rrs);

    let authority_rrs = u16::from_be_bytes([next(), next()]);
    println!("Authority RRs: {}", authority_rrs);

    let aditional_rrs = u16::from_be_bytes([next(), next()]);
    println!("Additional RRs: {}", aditional_rrs);

    // first segment
    assert_eq!(next(), 4); // len
    assert_eq!(next(), b'd');
    assert_eq!(next(), b'o');
    assert_eq!(next(), b'c');
    assert_eq!(next(), b's');
    // last segment
    assert_eq!(next(), 2); // len
    assert_eq!(next(), b'r');
    assert_eq!(next(), b's');
    assert_eq!(next(), 0); // null term

    // type A
    assert_eq!(next(), 0x00);
    assert_eq!(next(), 0x01);

    // class IN
    assert_eq!(next(), 0x00);
    assert_eq!(next(), 0x01);

    // name bytes
    next();
    next();

    // type A (in answer)
    assert_eq!(next(), 0x00);
    assert_eq!(next(), 0x01);

    // class IN (in answer)
    assert_eq!(next(), 0x00);
    assert_eq!(next(), 0x01);

    // time to live
    let ttl = u32::from_be_bytes([next(), next(), next(), next()]);
    println!("Time to live: {}s", ttl);

    // this is bad to assume IPv4
    let resp_len = u16::from_be_bytes([next(), next()]);
    assert_eq!(resp_len, 4);

    let docs_rs_ip: Ipv4Addr = Ipv4Addr::new(next(), next(), next(), next());

    println!("docs.rs IP address: {}", docs_rs_ip);
}
