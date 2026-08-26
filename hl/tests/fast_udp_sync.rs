//! Exact SPI transaction assertions for the *sync* fast UDP path.
//!
//! `FastUdp` and `FastUdpAsync` are generated from one source by
//! `maybe-async-cfg` (see `hl/src/fast_udp.rs`), so a change to that source —
//! a bad `idents` remap, or the `yield_once`/`yield_once_async` split — could
//! break only the sync half while the async half (exercised by
//! `hl/tests/fast_udp_mock.rs`, gated on the `eha1` feature) keeps passing.
//! Deliberately un-gated: this file must run with no features enabled at all,
//! so the sync `FastUdp` has real test coverage rather than relying on the
//! `no_run` doctest to catch a compile error.

use ehm::eh1::spi::{Mock as SpiMock, Transaction as SpiTransaction};
use w5500_hl::fast_udp::{FastUdp, UdpFrame};
use w5500_ll::{
    Sn, SnReg,
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

/// A successful `udp_recv_exact` over the sync `FastUdp`: one combined
/// `Sn_RX_RSR`/`Sn_RX_RD` read, one buffer read carrying header and payload
/// together, the read pointer advance, and the RECV command.
#[test]
fn recv_exact_reads_a_full_frame() {
    const SOCKET: Sn = Sn::Sn6;
    const READ_POINTER: u16 = 0x1234;
    const PAYLOAD_LEN: u16 = 180;
    const FRAME_LEN: u16 = 8 + PAYLOAD_LEN;

    let mut frame_bytes: Vec<u8> = vec![192, 168, 0, 1, 0xC0, 0x30];
    frame_bytes.extend(PAYLOAD_LEN.to_be_bytes());
    frame_bytes.extend((0..PAYLOAD_LEN).map(|index| index as u8));

    let mut expectations: Vec<SpiTransaction<u8>> = Vec::new();
    // One combined read of Sn_RX_RSR and Sn_RX_RD (they are adjacent).
    let mut pointers: Vec<u8> = FRAME_LEN.to_be_bytes().to_vec();
    pointers.extend(READ_POINTER.to_be_bytes());
    expectations.extend(read_transaction(
        SnReg::RX_RSR0.addr(),
        SOCKET.block(),
        pointers,
    ));
    // The single buffer read: header and payload in one transaction.
    expectations.extend(read_transaction(
        READ_POINTER,
        SOCKET.rx_block(),
        frame_bytes.clone(),
    ));
    // Advance the read pointer past the whole frame.
    expectations.extend(write_transaction(
        SnReg::RX_RD0.addr(),
        SOCKET.block(),
        READ_POINTER.wrapping_add(FRAME_LEN).to_be_bytes().to_vec(),
    ));
    // RECV command.
    expectations.extend(write_transaction(
        SnReg::CR.addr(),
        SOCKET.block(),
        vec![w5500_ll::SocketCommand::Recv.into()],
    ));

    let mut w5500 = W5500::new(SpiMock::new(&expectations));
    let mut frame: UdpFrame<188> = UdpFrame::new();

    w5500.udp_recv_exact(SOCKET, &mut frame).unwrap();

    assert_eq!(frame.payload(), &frame_bytes[8..]);
    // Mock::done() fails if any expectation went unused or any extra
    // transaction was issued.
    w5500.free().done();
}

/// `udp_send_exact` must refuse a payload that does not fit in the free TX
/// space, rather than transmitting it truncated.
#[test]
fn send_exact_refuses_when_tx_space_is_insufficient() {
    const SOCKET: Sn = Sn::Sn6;
    let payload: [u8; 32] = [0; 32];

    let mut expectations: Vec<SpiTransaction<u8>> = Vec::new();
    // Only 16 bytes free for a 32-byte datagram.
    let mut pointers: Vec<u8> = 16u16.to_be_bytes().to_vec();
    pointers.extend(0u16.to_be_bytes());
    pointers.extend(0x0500u16.to_be_bytes());
    expectations.extend(read_transaction(
        SnReg::TX_FSR0.addr(),
        SOCKET.block(),
        pointers,
    ));
    // Deliberately no buffer write, no TX_WR write, and no CR write in the
    // expectation list: `Mock::done()` below fails if the driver issues any
    // of them, which is exactly what transmitting the truncated payload
    // would do.

    let mut w5500 = W5500::new(SpiMock::new(&expectations));

    let result = w5500.udp_send_exact(SOCKET, &payload);

    assert!(
        matches!(result, Err(w5500_hl::Error::OutOfMemory)),
        "got {result:?}"
    );
    w5500.free().done();
}
