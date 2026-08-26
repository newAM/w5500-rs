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
