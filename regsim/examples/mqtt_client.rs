//! This is a simulation of the W5500 registers that runs on your local machine.
//!
//! This does not require any embedded hardware to use.
//!
//! This code is very minimal to make this example readable.
//! Do as I say, not as I do: Hard coded MQTT is bad.
//!
//! **Note:** This will communicate with external network services.

use core::panic;
use std::{
    net::{SocketAddr, ToSocketAddrs},
    thread::sleep,
    time::Duration,
};

use w5500_hl::Tcp;
use w5500_ll::{Registers, Sn, SocketInterrupt, VERSION, net::SocketAddrV4};
use w5500_regsim::W5500;

// socket to use for the MQTT client, any socket will work
const MQTT_SOCKET: Sn = Sn::Sn0;
// this is unused in the register simulation
const MQTT_SOURCE_PORT: u16 = 33650;
// hard-coded MQTT CONNECT packet
const MQTT_CONNECT: [u8; 14] = [
    0x10, 0x0C, 0x00, 0x04, b'M', b'Q', b'T', b'T', 0x04, 0x02, 0x0E, 0x10, 0x00, 0x00,
];

fn main() {
    // this is for register simulation logging
    stderrlog::new()
        .verbosity(4)
        .timestamp(stderrlog::Timestamp::Nanosecond)
        .init()
        .unwrap();

    // in a real embedded system there is a lot more boilerplate such as:
    // - DHCP (or setting a static IP)
    // - setting a valid EUI-48 MAC address
    // - Checking link up at the physical layer
    //
    // the register simulation allows us to cheat a little since your PC
    // (hopefully) already has a valid IP/MAC/Gateway/subnet mask

    let mqtt_server: SocketAddrV4 = "broker.hivemq.com.:443"
        .to_socket_addrs()
        .expect("Failed to resolve address for broker.hivemq.com")
        .find_map(|addr| match addr {
            SocketAddr::V4(v4) => Some(v4),
            SocketAddr::V6(_) => None,
        })
        .expect("Failed to resolve an IPv4 address for broker.hivemq.com");

    let mut w5500: W5500 = W5500::default();
    // sanity check
    assert_eq!(w5500.version().unwrap(), VERSION);

    // start the 3-way handshake
    w5500
        .tcp_connect(MQTT_SOCKET, MQTT_SOURCE_PORT, &mqtt_server)
        .expect("Failed to initiate 3-way handshake");

    // wait for the CON interrupt, indicating the socket connection is now
    // established
    loop {
        let sn_ir = w5500.sn_ir(MQTT_SOCKET).unwrap();
        if sn_ir.con_raised() {
            break;
        }
        if sn_ir.discon_raised() | sn_ir.timeout_raised() {
            panic!("Failed to connect");
        }
        sleep(Duration::from_millis(100));
    }

    // clear the CON interrupt
    w5500
        .set_sn_ir(MQTT_SOCKET, SocketInterrupt::CON_MASK)
        .unwrap();

    // send the CONNECT packet
    let tx_bytes = w5500
        .tcp_write(MQTT_SOCKET, &MQTT_CONNECT)
        .expect("Failed to send CONNECT");
    assert_eq!(usize::from(tx_bytes), MQTT_CONNECT.len());

    // wait for the RECV interrupt, indicating there is data to read
    loop {
        let sn_ir = w5500.sn_ir(MQTT_SOCKET).unwrap();
        if sn_ir.recv_raised() {
            break;
        }
        if sn_ir.discon_raised() | sn_ir.timeout_raised() {
            panic!("Socket disconnected while waiting for RECV");
        }
        sleep(Duration::from_millis(100));
    }

    // clear the RECV interrupt
    w5500
        .set_sn_ir(MQTT_SOCKET, SocketInterrupt::RECV_MASK)
        .unwrap();

    // read the response, this should be a 4-byte CONNACK response
    let mut buf = [0; 10];
    let rx_bytes = w5500
        .tcp_read(MQTT_SOCKET, &mut buf)
        .expect("Failed to read CONNACK");
    let filled_buf = &buf[..rx_bytes.into()];

    // check the received packet is a CONNACK
    assert_eq!(filled_buf[0], 2 << 4);
    // check that the connection code is ACCEPT
    assert_eq!(filled_buf[3], 0);

    // the rest is up to you, once connected you can publish message,
    // or subscribe to topics
}
