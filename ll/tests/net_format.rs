use w5500_ll::net::{Eui48Addr, Ipv4Addr, SocketAddrV4};

const MAC_FMT_CASES: &[(Eui48Addr, &str)] = &[
    (
        Eui48Addr::new(0x01, 0x23, 0x45, 0x67, 0x89, 0xAB),
        "01:23:45:67:89:AB",
    ),
    (Eui48Addr::UNSPECIFIED, "00:00:00:00:00:00"),
];

#[test]
fn mac_format() {
    for (mac, expected) in MAC_FMT_CASES {
        assert_eq!(format!("{mac}"), *expected);
    }
}

#[test]
fn mac_uformat() {
    for (mac, expected) in MAC_FMT_CASES {
        let mut s = String::new();
        ufmt::uwrite!(s, "{}", mac).unwrap();
        assert_eq!(s, *expected);
    }
}

const IPV4_FMT_CASES: &[(Ipv4Addr, &str)] = &[
    (Ipv4Addr::new(1, 2, 3, 4), "1.2.3.4"),
    (Ipv4Addr::BROADCAST, "255.255.255.255"),
    (Ipv4Addr::UNSPECIFIED, "0.0.0.0"),
];

#[test]
fn ipv4_format() {
    for (ip, expected) in IPV4_FMT_CASES {
        assert_eq!(format!("{ip}"), *expected);
    }
}

#[test]
fn ipv4_uformat() {
    for (ip, expected) in IPV4_FMT_CASES {
        let mut s = String::new();
        ufmt::uwrite!(s, "{}", ip).unwrap();
        assert_eq!(s, *expected);
    }
}

const SOCKET_ADDR_FMT_CASES: &[(SocketAddrV4, &str)] = &[
    (SocketAddrV4::new(Ipv4Addr::new(1, 2, 3, 4), 60), "1.2.3.4:60"),
    (SocketAddrV4::new(Ipv4Addr::BROADCAST, u16::MAX), "255.255.255.255:65535"),
    (SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0), "0.0.0.0:0"),
];

#[test]
fn socket_addr_format() {
    for (addr, expected) in SOCKET_ADDR_FMT_CASES {
        assert_eq!(format!("{addr}"), *expected);
    }
}


#[test]
fn socket_addr_uformat() {
    for (addr, expected) in SOCKET_ADDR_FMT_CASES {
        let mut s = String::new();
        ufmt::uwrite!(s, "{}", addr).unwrap();
        assert_eq!(s, *expected);
    }
}
