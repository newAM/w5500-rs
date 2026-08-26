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

#[tokio::test(flavor = "current_thread")]
async fn async_trait_is_callable() {
    const SOCKET: Sn = Sn::Sn6;
    let expectations = read_transaction(
        SnReg::SR.addr(),
        SOCKET.block(),
        vec![SocketStatus::Udp.into()],
    );
    let mut w5500 = W5500::new(SpiMock::new(&expectations));

    let status = w5500.udp_socket_status(SOCKET).await.unwrap();

    assert_eq!(status, Ok(SocketStatus::Udp));
    w5500.free().done();
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
