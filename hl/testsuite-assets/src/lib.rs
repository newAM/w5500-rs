//! Shared on-target testing assets.
//!
//! This is used to keep the IP addresses in sync.

#![no_std]

use w5500_hl::ll::Sn;
use w5500_hl::net::{Eui48Addr, Ipv4Addr, SocketAddrV4};

/// W5500 MAC.
///
/// MAC addresses starting with `0x02` are reserved for testing.
pub const MAC: Eui48Addr = Eui48Addr::new(0x02, 0x03, 0x04, 0x05, 0x06, 0x07);
/// W5500 static IPv4
pub const W5500_IP: Ipv4Addr = Ipv4Addr::new(10, 0, 0, 50);
/// W5500 gateway IP
pub const GATEWAY: Ipv4Addr = Ipv4Addr::new(10, 0, 0, 1);
/// W5500 subnet mask
pub const SUBNET_MASK: Ipv4Addr = Ipv4Addr::new(255, 255, 255, 0);

/// Peer IP address
///
/// This is the IPv4 for the remote peer that the W5500 will connect with for
/// testing.
pub const PEER_IP: Ipv4Addr = Ipv4Addr::new(10, 0, 0, 3);

/// Socket to use for HTTP server tests.
pub const HTTP_SOCKET: Sn = Sn::Sn7;
/// Port to serve HTTP on (served by the W5500).
pub const W5500_HTTP_PORT: u16 = 80;

/// Socket to use for non-HTTP tests using TCP.
pub const TCP_SOCKET: Sn = Sn::Sn6;
/// Peer port to use for TCP tests.
pub const PEER_TCP_PORT: u16 = 8080;
/// Peer address for TCP tests.
pub const PEER_TCP_ADDR: SocketAddrV4 = SocketAddrV4::new(PEER_IP, PEER_TCP_PORT);
/// W5500 port to use for TCP tests.
pub const W5500_TCP_PORT: u16 = 8123;
/// W5500 address for TCP tests.
pub const W5500_TCP_ADDR: SocketAddrV4 = SocketAddrV4::new(W5500_IP, W5500_TCP_PORT);
/// Number of chunks for the TCP test.
pub const NUM_CHUNKS: usize = 32;
/// Chunk size for the TCP test.
pub const CHUNK_SIZE: usize = 1234;

/// Socket to use for tests using UDP.
pub const UDP_SOCKET: Sn = Sn::Sn5;
/// Peer port for the for UDP tests.
pub const PEER_UDP_PORT: u16 = 5657;
/// Peer address for UDP tests.
pub const PEER_UDP_ADDR: SocketAddrV4 = SocketAddrV4::new(PEER_IP, PEER_UDP_PORT);
/// W5500 port to use for UDP tests.
pub const W5500_UDP_PORT: u16 = 5656;
/// W5500 address for TCP tests.
pub const W5500_UDP_ADDR: SocketAddrV4 = SocketAddrV4::new(W5500_IP, W5500_UDP_PORT);
/// Test data for UDP tests.
pub const UDP_DATA: &[u8] = b"hi";
