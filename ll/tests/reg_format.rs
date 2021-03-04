use w5500_ll::{Mode, PhyCfg, SocketInterrupt};

#[test]
fn sn_ir_none() {
    let snir = SocketInterrupt::default();
    assert_eq!(
        format!("{:#}", snir),
        r#"SocketInterrupt {
    con_raised: false,
    discon_raised: false,
    recv_raised: false,
    timeout_raised: false,
    sendok_raised: false,
}"#
    );
}

#[test]
fn sn_ir_all() {
    let snir: SocketInterrupt = u8::MAX.into();

    assert_eq!(
        format!("{:#}", snir),
        r#"SocketInterrupt {
    con_raised: true,
    discon_raised: true,
    recv_raised: true,
    timeout_raised: true,
    sendok_raised: true,
}"#
    );
}

#[test]
fn sn_ir_partial() {
    let snir: SocketInterrupt = SocketInterrupt::DISCON_MASK.into();

    assert_eq!(
        format!("{:#}", snir),
        r#"SocketInterrupt {
    con_raised: false,
    discon_raised: true,
    recv_raised: false,
    timeout_raised: false,
    sendok_raised: false,
}"#
    );
}

#[test]
fn mode() {
    let mode: Mode = Mode::default();

    assert_eq!(
        format!("{:#}", mode),
        r#"Mode {
    wol_enabled: false,
    pb_enabled: false,
    pppoe_enabled: false,
    farp_enabled: false,
}"#,
    )
}

#[test]
fn phy_cfg() {
    let phy_cfg: PhyCfg = PhyCfg::default();

    assert_eq!(
        format!("{:#}", phy_cfg),
        r#"PhyCfg {
    opmd: Ok(
        Auto,
    ),
    dpx: Half,
    spd: Mbps10,
    lnk: Down,
}"#,
    )
}
