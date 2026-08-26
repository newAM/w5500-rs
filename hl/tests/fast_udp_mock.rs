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

/// Builds the expectations for a successful 188-byte receive.
///
/// Returns the expectation list and the payload bytes it will deliver.
fn recv_exact_expectations(
    socket: Sn,
    read_pointer: u16,
    header_payload_len: u16,
    payload_len: usize,
) -> (Vec<SpiTransaction<u8>>, Vec<u8>, usize) {
    let frame_len: u16 = 8 + header_payload_len;
    // Counts reads addressed at the socket RX buffer block. This is the
    // quantity optimization O1 is about.
    let mut rx_buffer_reads: usize = 0;

    let mut frame_bytes: Vec<u8> = vec![192, 168, 0, 1, 0xC0, 0x30];
    frame_bytes.extend(header_payload_len.to_be_bytes());
    frame_bytes.extend((0..payload_len).map(|index| index as u8));

    let mut expectations: Vec<SpiTransaction<u8>> = Vec::new();
    // 1. One combined read of Sn_RX_RSR and Sn_RX_RD (they are adjacent).
    let mut pointers: Vec<u8> = (8 + header_payload_len).to_be_bytes().to_vec();
    pointers.extend(read_pointer.to_be_bytes());
    expectations.extend(read_transaction(
        SnReg::RX_RSR0.addr(),
        socket.block(),
        pointers,
    ));
    // 2. THE single buffer read: header and payload in one transaction.
    expectations.extend(read_transaction(
        read_pointer,
        socket.rx_block(),
        frame_bytes.clone(),
    ));
    rx_buffer_reads += 1;
    // 3. Advance the read pointer past the whole frame.
    expectations.extend(write_transaction(
        SnReg::RX_RD0.addr(),
        socket.block(),
        read_pointer.wrapping_add(frame_len).to_be_bytes().to_vec(),
    ));
    // 4. RECV command.
    expectations.extend(write_transaction(
        SnReg::CR.addr(),
        socket.block(),
        vec![SocketCommand::Recv.into()],
    ));

    (expectations, frame_bytes[8..].to_vec(), rx_buffer_reads)
}

#[tokio::test(flavor = "current_thread")]
async fn recv_exact_issues_one_buffer_read() {
    const SOCKET: Sn = Sn::Sn6;
    const READ_POINTER: u16 = 0x1234;

    let (expectations, expected_payload, rx_buffer_reads) =
        recv_exact_expectations(SOCKET, READ_POINTER, 180, 180);

    // The proof of optimization O1: exactly ONE read addressed at the socket RX
    // buffer block, carrying header and payload together. The generic path
    // issues two — see `recv_from_generic_issues_two_buffer_reads`.
    //
    // This assertion alone is not the proof: it describes the expectation list.
    // `Mock::done()` below is what proves the driver issued exactly these
    // transactions and no others.
    assert_eq!(
        rx_buffer_reads, 1,
        "receive must fetch header and payload in one SPI transaction"
    );

    let mut w5500 = W5500::new(SpiMock::new(&expectations));
    let mut frame: UdpFrame<188> = UdpFrame::new();

    w5500.udp_recv_exact(SOCKET, &mut frame).await.unwrap();

    assert_eq!(
        frame.origin(),
        SocketAddrV4::new(Ipv4Addr::new(192, 168, 0, 1), 49200)
    );
    assert_eq!(frame.payload(), expected_payload.as_slice());
    // Mock::done() fails if any expectation went unused, so this also asserts
    // that no additional SPI transaction was issued.
    w5500.free().done();
}

/// `rsr` below the header length means nothing has arrived: one transaction,
/// then WouldBlock.
#[tokio::test(flavor = "current_thread")]
async fn recv_exact_would_block_when_buffer_is_empty() {
    const SOCKET: Sn = Sn::Sn6;
    for received_size in [0u16, 1, 7] {
        let mut pointers: Vec<u8> = received_size.to_be_bytes().to_vec();
        pointers.extend(0x1234u16.to_be_bytes());
        let expectations = read_transaction(SnReg::RX_RSR0.addr(), SOCKET.block(), pointers);

        let mut w5500 = W5500::new(SpiMock::new(&expectations));
        let mut frame: UdpFrame<188> = UdpFrame::new();

        let result = w5500.udp_recv_exact(SOCKET, &mut frame).await;

        assert!(
            matches!(result, Err(w5500_hl::Error::WouldBlock)),
            "rsr={received_size} should be WouldBlock, got {result:?}"
        );
        w5500.free().done();
    }
}

/// The header has arrived but the payload has not: still WouldBlock, and the
/// read pointer must not move.
#[tokio::test(flavor = "current_thread")]
async fn recv_exact_would_block_while_datagram_is_still_arriving() {
    const SOCKET: Sn = Sn::Sn6;
    const READ_POINTER: u16 = 0x1234;

    for received_size in [8u16, 100, 187] {
        let mut expectations: Vec<SpiTransaction<u8>> = Vec::new();
        let mut pointers: Vec<u8> = received_size.to_be_bytes().to_vec();
        pointers.extend(READ_POINTER.to_be_bytes());
        expectations.extend(read_transaction(
            SnReg::RX_RSR0.addr(),
            SOCKET.block(),
            pointers,
        ));
        // Cold path reads only the 8-byte header to disambiguate.
        let mut header: Vec<u8> = vec![192, 168, 0, 1, 0xC0, 0x30];
        header.extend(180u16.to_be_bytes());
        expectations.extend(read_transaction(READ_POINTER, SOCKET.rx_block(), header));

        let mut w5500 = W5500::new(SpiMock::new(&expectations));
        let mut frame: UdpFrame<188> = UdpFrame::new();

        let result = w5500.udp_recv_exact(SOCKET, &mut frame).await;

        assert!(
            matches!(result, Err(w5500_hl::Error::WouldBlock)),
            "rsr={received_size} should be WouldBlock, got {result:?}"
        );
        w5500.free().done();
    }
}

/// A datagram longer than the frame must be rejected AND consumed, so the
/// socket is not wedged by one malformed packet.
#[tokio::test(flavor = "current_thread")]
async fn recv_exact_rejects_and_discards_an_oversized_datagram() {
    const SOCKET: Sn = Sn::Sn6;
    const READ_POINTER: u16 = 0x1234;

    // A header claiming 181 bytes, arriving in a 188-byte frame. `rsr` is 189,
    // so the hot path runs and reads a full 188-byte frame, but the header
    // disagrees with the caller's fixed size.
    //
    // The helper already derives the pointer advance from the *header's* length
    // (8 + 181 = 189), which is exactly the behaviour under test: the driver
    // must skip what the datagram actually occupies, not what the caller wanted.
    let (expectations, _, _) = recv_exact_expectations(SOCKET, READ_POINTER, 181, 180);

    let mut w5500 = W5500::new(SpiMock::new(&expectations));
    let mut frame: UdpFrame<188> = UdpFrame::new();

    let result = w5500.udp_recv_exact(SOCKET, &mut frame).await;

    assert!(
        matches!(
            result,
            Err(w5500_hl::Error::UnexpectedLength {
                expected: 180,
                received: 181
            })
        ),
        "got {result:?}"
    );
    // done() proves RX_RD advanced and RECV was issued despite the error.
    w5500.free().done();
}

/// A datagram shorter than the frame, fully arrived, is also a length error —
/// and must likewise be consumed.
#[tokio::test(flavor = "current_thread")]
async fn recv_exact_rejects_and_discards_an_undersized_datagram() {
    const SOCKET: Sn = Sn::Sn6;
    const READ_POINTER: u16 = 0x1234;
    const SHORT_PAYLOAD_LEN: u16 = 179;

    let mut expectations: Vec<SpiTransaction<u8>> = Vec::new();
    let mut pointers: Vec<u8> = (8 + SHORT_PAYLOAD_LEN).to_be_bytes().to_vec();
    pointers.extend(READ_POINTER.to_be_bytes());
    expectations.extend(read_transaction(
        SnReg::RX_RSR0.addr(),
        SOCKET.block(),
        pointers,
    ));
    let mut header: Vec<u8> = vec![192, 168, 0, 1, 0xC0, 0x30];
    header.extend(SHORT_PAYLOAD_LEN.to_be_bytes());
    expectations.extend(read_transaction(READ_POINTER, SOCKET.rx_block(), header));
    expectations.extend(write_transaction(
        SnReg::RX_RD0.addr(),
        SOCKET.block(),
        READ_POINTER
            .wrapping_add(8 + SHORT_PAYLOAD_LEN)
            .to_be_bytes()
            .to_vec(),
    ));
    expectations.extend(write_transaction(
        SnReg::CR.addr(),
        SOCKET.block(),
        vec![SocketCommand::Recv.into()],
    ));

    let mut w5500 = W5500::new(SpiMock::new(&expectations));
    let mut frame: UdpFrame<188> = UdpFrame::new();

    let result = w5500.udp_recv_exact(SOCKET, &mut frame).await;

    assert!(
        matches!(
            result,
            Err(w5500_hl::Error::UnexpectedLength {
                expected: 180,
                received: 179
            })
        ),
        "got {result:?}"
    );
    w5500.free().done();
}
