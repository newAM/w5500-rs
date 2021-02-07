use w5500_ll::SocketInterrupt;

#[test]
fn sn_ir_none() {
    let snir = SocketInterrupt::default();
    assert_eq!(
        format!("{}", snir),
        r#"Socket interrupts:
CON: not raised
DISCON: not raised
RECV: not raised
SENDOK: not raised
"#
    );
}

#[test]
fn sn_ir_all() {
    let snir: SocketInterrupt = u8::MAX.into();

    assert_eq!(
        format!("{}", snir),
        r#"Socket interrupts:
CON: raised
DISCON: raised
RECV: raised
SENDOK: raised
"#
    );
}

#[test]
fn sn_ir_partial() {
    let snir: SocketInterrupt = SocketInterrupt::DISCON_MASK.into();

    assert_eq!(
        format!("{}", snir),
        r#"Socket interrupts:
CON: not raised
DISCON: raised
RECV: not raised
SENDOK: not raised
"#
    );
}
