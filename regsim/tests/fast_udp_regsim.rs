//! End-to-end validation of the async fast path against the register
//! simulator, over real `std::net` UDP sockets on the host.
//!
//! `w5500_regsim::W5500` implements `w5500_ll::aio::Registers` directly, so the
//! async implementation is exercised without a shim.

use std::net::UdpSocket;

use w5500_hl::{
    Error,
    fast_udp::{FastUdpAsync, UdpFrame},
};
use w5500_ll::{
    Sn,
    net::{Ipv4Addr, SocketAddrV4},
};

const SOCKET: Sn = Sn::Sn6;
// Chosen below Windows's dynamic/ephemeral port range (49152-65535, and on
// this host additionally excluded 49152-49251 by Hyper-V/WSL NAT) so the
// literal `Sn_PORT` bind can never collide with an OS-assigned ephemeral
// port for `peer_socket`'s `bind("127.0.0.1:0")`.
const DEVICE_PORT: u16 = 15201;
const PAYLOAD_LEN: usize = 180;
const FRAME_LEN: usize = 188;

#[tokio::test(flavor = "current_thread")]
async fn receives_a_real_datagram_and_replies() {
    let mut w5500 = w5500_regsim::W5500::default();

    // The host end, standing in for the flight simulator.
    let peer_socket = UdpSocket::bind("127.0.0.1:0").expect("bind host socket");
    let peer_address = match peer_socket.local_addr().expect("host addr") {
        std::net::SocketAddr::V4(address) => address,
        other => panic!("expected IPv4, got {other:?}"),
    };
    let peer = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), peer_address.port());

    w5500
        .udp_bind_to_peer(SOCKET, DEVICE_PORT, &peer)
        .await
        .expect("bind");

    // Nothing sent yet: the poll must be cheap and non-fatal.
    let mut frame: UdpFrame<FRAME_LEN> = UdpFrame::new();
    assert!(
        matches!(
            w5500.udp_recv_exact(SOCKET, &mut frame).await,
            Err(Error::WouldBlock)
        ),
        "an empty socket must report WouldBlock"
    );

    // Send a real datagram from the host.
    let mut outbound: [u8; PAYLOAD_LEN] = [0; PAYLOAD_LEN];
    for (index, byte) in outbound.iter_mut().enumerate() {
        *byte = index as u8;
    }
    peer_socket
        .send_to(&outbound, format!("127.0.0.1:{DEVICE_PORT}"))
        .expect("host send");

    // Poll until it lands. Bounded so a failure is a test failure, not a hang.
    let mut attempts_remaining = 1000;
    loop {
        match w5500.udp_recv_exact(SOCKET, &mut frame).await {
            Ok(()) => break,
            Err(Error::WouldBlock) => {
                attempts_remaining -= 1;
                assert!(attempts_remaining > 0, "datagram never arrived");
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
            Err(other) => panic!("receive failed: {other:?}"),
        }
    }

    assert_eq!(frame.payload(), outbound.as_slice());
    assert_eq!(frame.origin().port(), peer.port());

    // Reply with 32 bytes of little-endian f32, as the rig does.
    let commands: [f32; 8] = [1.0, 2.0, 0.5, -1.0, 0.0, 100.0, -0.25, 3.5];
    let mut reply: [u8; 32] = [0; 32];
    for (index, command) in commands.iter().enumerate() {
        reply[index * 4..index * 4 + 4].copy_from_slice(&command.to_le_bytes());
    }
    w5500.udp_send_exact(SOCKET, &reply).await.expect("send");

    peer_socket
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .expect("set timeout");
    let mut inbound: [u8; 32] = [0; 32];
    let (received_len, _) = peer_socket.recv_from(&mut inbound).expect("host recv");

    assert_eq!(received_len, 32);
    assert_eq!(inbound, reply);
    // Decode on the host and confirm the values survived the round trip.
    for (index, expected) in commands.iter().enumerate() {
        let mut word: [u8; 4] = [0; 4];
        word.copy_from_slice(&inbound[index * 4..index * 4 + 4]);
        assert_eq!(f32::from_le_bytes(word), *expected, "command {index}");
    }
}

/// A datagram of the wrong length must be rejected and consumed, leaving the
/// socket able to receive the next one.
#[tokio::test(flavor = "current_thread")]
async fn wrong_length_datagram_does_not_wedge_the_socket() {
    let mut w5500 = w5500_regsim::W5500::default();
    const PORT: u16 = 15202; // see the DEVICE_PORT comment above

    let peer_socket = UdpSocket::bind("127.0.0.1:0").expect("bind host socket");
    let peer_port = peer_socket.local_addr().expect("addr").port();
    let peer = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), peer_port);

    w5500
        .udp_bind_to_peer(SOCKET, PORT, &peer)
        .await
        .expect("bind");

    let mut frame: UdpFrame<FRAME_LEN> = UdpFrame::new();

    // Send ONLY the short datagram first, and poll until the cold path
    // (`recv_resync_short`) actually reports it. This is a deliberate
    // synchronization point: `udp_recv_exact` only takes the cold path while
    // `Sn_RX_RSR < FRAME_LEN` (188). If the correct 180-byte datagram were
    // sent up front too, both would typically land before the first poll,
    // `Sn_RX_RSR` would already be 375 (>= 188), and the very first call
    // would take the *hot* path instead -- re-deriving the header at the
    // still-unadvanced `Sn_RX_RD` and performing its own consume. That hot
    // path would silently mask a broken cold path: both the length error and
    // the eventual success would be observed regardless of whether
    // `recv_resync_short` itself ever advanced `Sn_RX_RD` / issued `RECV`.
    // Waiting here for the length error to be observed guarantees the cold
    // path ran on its own and is the only thing that could have consumed the
    // short datagram.
    peer_socket
        .send_to(&[0u8; 179], format!("127.0.0.1:{PORT}"))
        .expect("send short");

    let mut short_datagram_attempts_remaining = 2000;
    loop {
        match w5500.udp_recv_exact(SOCKET, &mut frame).await {
            Err(Error::UnexpectedLength { expected, received }) => {
                assert_eq!(expected, 180);
                assert_eq!(received, 179);
                break;
            }
            Err(Error::WouldBlock) => {
                short_datagram_attempts_remaining -= 1;
                assert!(
                    short_datagram_attempts_remaining > 0,
                    "the short datagram never arrived"
                );
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
            other => panic!("unexpected result while awaiting the short datagram: {other:?}"),
        }
    }

    // Only now, after the cold path has run and (if the driver is correct)
    // consumed the short datagram on its own, send the correct-length
    // datagram and confirm the socket was left usable.
    peer_socket
        .send_to(&[0u8; 180], format!("127.0.0.1:{PORT}"))
        .expect("send correct");

    let mut correct_datagram_attempts_remaining = 2000;
    loop {
        match w5500.udp_recv_exact(SOCKET, &mut frame).await {
            Ok(()) => break,
            Err(Error::WouldBlock) => {
                correct_datagram_attempts_remaining -= 1;
                assert!(
                    correct_datagram_attempts_remaining > 0,
                    "the socket wedged: the 180-byte datagram never arrived"
                );
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
            Err(other) => {
                panic!("unexpected result while awaiting the correct datagram: {other:?}")
            }
        }
    }
}
