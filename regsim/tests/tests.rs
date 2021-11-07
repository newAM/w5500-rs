use w5500_ll::{Reg, Registers, Sn, SnReg, COMMON_BLOCK_OFFSET, VERSION};
use w5500_regsim::W5500;

#[test]
#[should_panic]
fn invalid_block() {
    let mut w5500 = W5500::default();
    w5500.write(SnReg::SR.addr(), 0x04, &[0]).ok();
}

#[test]
fn reg_versionr() {
    let mut buf: [u8; 1] = [0x00];
    let mut w5500 = W5500::default();
    w5500
        .read(Reg::VERSIONR.addr(), COMMON_BLOCK_OFFSET, &mut buf)
        .unwrap();
    assert_eq!(buf[0], VERSION);
    assert_eq!(w5500.version().unwrap(), VERSION);
}

#[test]
fn sn_tx_fsr_reset_value() {
    let mut w5500 = W5500::default();
    assert_eq!(w5500.sn_tx_fsr(Sn::Sn0).unwrap(), 0x0800);
}

#[test]
fn sn_frag_reset_value() {
    let mut w5500 = W5500::default();
    assert_eq!(w5500.sn_frag(Sn::Sn0).unwrap(), 0x4000);
}

#[test]
fn remove_me() {
    let mut w5500 = W5500::default();
    const ADDR: w5500_ll::net::SocketAddrV4 =
        w5500_ll::net::SocketAddrV4::new(w5500_ll::net::Ipv4Addr::new(192, 168, 3, 4), 0x1234);
    w5500.set_sn_dest(Sn::Sn0, &ADDR).unwrap();
    assert_eq!(ADDR, w5500.sn_dest(Sn::Sn0).unwrap())
}
