use w5500_ll::net::Eui48Addr;

#[test]
fn mac_format() {
    assert_eq!(
        format!("{}", Eui48Addr::new(0x01, 0x23, 0x45, 0x67, 0x89, 0xAB)),
        "01:23:45:67:89:AB"
    );
    assert_eq!(format!("{}", Eui48Addr::UNSPECIFIED), "00:00:00:00:00:00")
}
