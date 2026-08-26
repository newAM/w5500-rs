//! Exact SPI transaction assertions for the fast UDP path.
#![cfg(feature = "eha1")]

use ehm::eh1::spi::{Mock as SpiMock, Transaction as SpiTransaction};
use w5500_hl::fast_udp::FastUdpAsync;
use w5500_ll::{
    Sn, SnReg, SocketStatus,
    eh1::vdm::W5500,
    spi::{AccessMode, vdm_header},
};

/// Builds the four mock expectations for one VDM read transaction.
fn read_transaction(address: u16, block: u8, response: Vec<u8>) -> Vec<SpiTransaction<u8>> {
    vec![
        SpiTransaction::transaction_start(),
        SpiTransaction::write_vec(vdm_header(address, block, AccessMode::Read).to_vec()),
        SpiTransaction::read_vec(response),
        SpiTransaction::transaction_end(),
    ]
}

/// Builds the four mock expectations for one VDM write transaction.
fn write_transaction(address: u16, block: u8, payload: Vec<u8>) -> Vec<SpiTransaction<u8>> {
    vec![
        SpiTransaction::transaction_start(),
        SpiTransaction::write_vec(vdm_header(address, block, AccessMode::Write).to_vec()),
        SpiTransaction::write_vec(payload),
        SpiTransaction::transaction_end(),
    ]
}

#[test]
fn unexpected_length_error_exists() {
    let error: w5500_hl::Error<core::convert::Infallible> = w5500_hl::Error::UnexpectedLength {
        expected: 180,
        received: 179,
    };
    match error {
        w5500_hl::Error::UnexpectedLength { expected, received } => {
            assert_eq!(expected, 180);
            assert_eq!(received, 179);
        }
        _ => panic!("wrong variant"),
    }
}

use w5500_hl::fast_udp::{UDP_FRAME_HEADER_LEN, UdpFrame};
use w5500_ll::net::{Ipv4Addr, SocketAddrV4};

#[test]
fn frame_reports_payload_length() {
    assert_eq!(UDP_FRAME_HEADER_LEN, 8);
    assert_eq!(UdpFrame::<188>::PAYLOAD_LEN, 180);

    let frame: UdpFrame<188> = UdpFrame::new();
    assert_eq!(frame.payload().len(), 180);
}

/// The W5500 receive header is big-endian (W5500 datasheet section 4.2).
///
/// The port and length bytes here are deliberately asymmetric: 0xC030 read as
/// little-endian would be 0x30C0 (12480, not 49200), and 0x00B4 would be
/// 0xB400 (46080, not 180). A `from_le_bytes` slip cannot pass this test.
#[test]
fn frame_decodes_big_endian_origin() {
    let mut frame: UdpFrame<188> = UdpFrame::new();
    frame.buffer_for_test()[..8].copy_from_slice(&[192, 168, 0, 1, 0xC0, 0x30, 0x00, 0xB4]);

    assert_eq!(
        frame.origin(),
        SocketAddrV4::new(Ipv4Addr::new(192, 168, 0, 1), 49200)
    );
}

#[test]
fn frame_payload_starts_after_the_header() {
    let mut frame: UdpFrame<12> = UdpFrame::new();
    frame.buffer_for_test().copy_from_slice(&[
        192, 168, 0, 1, 0xC0, 0x30, 0x00, 0x04, 0xDE, 0xAD, 0xBE, 0xEF,
    ]);

    assert_eq!(frame.payload(), &[0xDE, 0xAD, 0xBE, 0xEF]);
}

use w5500_ll::{Protocol, SocketCommand, SocketMode};

#[tokio::test(flavor = "current_thread")]
async fn bind_opens_a_udp_socket() {
    const SOCKET: Sn = Sn::Sn6;
    const PORT: u16 = 49200;

    let mut expectations: Vec<SpiTransaction<u8>> = Vec::new();
    // CLOSE, then one status read reporting Closed.
    expectations.extend(write_transaction(
        SnReg::CR.addr(),
        SOCKET.block(),
        vec![SocketCommand::Close.into()],
    ));
    expectations.extend(read_transaction(
        SnReg::SR.addr(),
        SOCKET.block(),
        vec![SocketStatus::Closed.into()],
    ));
    // Local port, big-endian.
    expectations.extend(write_transaction(
        SnReg::PORT0.addr(),
        SOCKET.block(),
        PORT.to_be_bytes().to_vec(),
    ));
    // Socket mode: protocol field = UDP. This is the hardware socket engine,
    // not MACRAW.
    expectations.extend(write_transaction(
        SnReg::MR.addr(),
        SOCKET.block(),
        vec![SocketMode::DEFAULT.set_protocol(Protocol::Udp).into()],
    ));
    // OPEN, then one status read reporting Udp.
    expectations.extend(write_transaction(
        SnReg::CR.addr(),
        SOCKET.block(),
        vec![SocketCommand::Open.into()],
    ));
    expectations.extend(read_transaction(
        SnReg::SR.addr(),
        SOCKET.block(),
        vec![SocketStatus::Udp.into()],
    ));

    let mut w5500 = W5500::new(SpiMock::new(&expectations));
    w5500.udp_bind(SOCKET, PORT).await.unwrap();
    w5500.free().done();
}

#[tokio::test(flavor = "current_thread")]
async fn bind_to_peer_also_writes_the_destination() {
    const SOCKET: Sn = Sn::Sn6;
    const PORT: u16 = 49200;
    const PEER: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(192, 168, 0, 1), 49200);

    let mut expectations: Vec<SpiTransaction<u8>> = Vec::new();
    expectations.extend(write_transaction(
        SnReg::CR.addr(),
        SOCKET.block(),
        vec![SocketCommand::Close.into()],
    ));
    expectations.extend(read_transaction(
        SnReg::SR.addr(),
        SOCKET.block(),
        vec![SocketStatus::Closed.into()],
    ));
    expectations.extend(write_transaction(
        SnReg::PORT0.addr(),
        SOCKET.block(),
        PORT.to_be_bytes().to_vec(),
    ));
    expectations.extend(write_transaction(
        SnReg::MR.addr(),
        SOCKET.block(),
        vec![SocketMode::DEFAULT.set_protocol(Protocol::Udp).into()],
    ));
    expectations.extend(write_transaction(
        SnReg::CR.addr(),
        SOCKET.block(),
        vec![SocketCommand::Open.into()],
    ));
    expectations.extend(read_transaction(
        SnReg::SR.addr(),
        SOCKET.block(),
        vec![SocketStatus::Udp.into()],
    ));
    // Destination IP and port, written once at bind time (optimization O3).
    //
    // `Registers::set_sn_dest` (ll/src/aio.rs) issues a single 6-byte write
    // starting at DIPR0 covering DIPR0..=DPORT1, not two separate writes.
    expectations.extend(write_transaction(
        SnReg::DIPR0.addr(),
        SOCKET.block(),
        vec![192, 168, 0, 1, 0xC0, 0x30],
    ));

    let mut w5500 = W5500::new(SpiMock::new(&expectations));
    w5500.udp_bind_to_peer(SOCKET, PORT, &PEER).await.unwrap();
    w5500.free().done();
}
