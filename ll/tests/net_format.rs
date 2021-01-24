use w5500_ll::net::{Eui48Addr, Ipv4Addr, SocketAddrV4};

#[test]
fn mac_format() {
    assert_eq!(
        format!("{}", Eui48Addr::new(0x01, 0x23, 0x45, 0x67, 0x89, 0xAB)),
        "01:23:45:67:89:AB"
    );
    assert_eq!(format!("{}", Eui48Addr::UNSPECIFIED), "00:00:00:00:00:00")
}

#[test]
fn ipv4_format() {
    assert_eq!(format!("{}", Ipv4Addr::new(1, 2, 3, 4)), "1.2.3.4");
    assert_eq!(format!("{}", Ipv4Addr::BROADCAST), "255.255.255.255");
    assert_eq!(format!("{}", Ipv4Addr::UNSPECIFIED), "0.0.0.0")
}

#[test]
fn socket_addr_format() {
    assert_eq!(
        format!("{}", SocketAddrV4::new(Ipv4Addr::new(1, 2, 3, 4), 60)),
        "1.2.3.4:60"
    );
    assert_eq!(
        format!("{}", SocketAddrV4::new(Ipv4Addr::BROADCAST, u16::MAX)),
        "255.255.255.255:65535"
    );
    assert_eq!(
        format!("{}", SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)),
        "0.0.0.0:0"
    )
}
