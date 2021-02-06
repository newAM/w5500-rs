#[cfg(feature = "std")]
const STD_IPV4: std::net::Ipv4Addr = std::net::Ipv4Addr::new(1, 2, 3, 4);
#[cfg(feature = "std")]
const STD_IP: std::net::IpAddr = std::net::IpAddr::V4(STD_IPV4);
#[cfg(feature = "std")]
const W5500_IPV4: w5500_ll::net::Ipv4Addr = w5500_ll::net::Ipv4Addr::new(1, 2, 3, 4);

#[test]
#[cfg(feature = "std")]
fn ip() {
    assert_eq!(w5500_ll::net::Ipv4Addr::from(STD_IPV4), W5500_IPV4);
    assert_eq!(std::net::Ipv4Addr::from(W5500_IPV4), STD_IPV4);
    assert_eq!(std::net::IpAddr::from(W5500_IPV4), STD_IP);

    assert_eq!(w5500_ll::net::Ipv4Addr::from(&STD_IPV4), W5500_IPV4);
    assert_eq!(std::net::Ipv4Addr::from(&W5500_IPV4), STD_IPV4);
    assert_eq!(std::net::IpAddr::from(&W5500_IPV4), STD_IP);
}

#[test]
#[cfg(feature = "std")]
fn socket() {
    let std_sv4: std::net::SocketAddrV4 = std::net::SocketAddrV4::new(STD_IPV4, 1234u16);
    let std_s: std::net::SocketAddr = std::net::SocketAddr::V4(std_sv4);
    let w5500_sv4: w5500_ll::net::SocketAddrV4 =
        w5500_ll::net::SocketAddrV4::new(W5500_IPV4, 1234u16);

    assert_eq!(w5500_ll::net::SocketAddrV4::from(std_sv4), w5500_sv4);
    assert_eq!(std::net::SocketAddrV4::from(w5500_sv4), std_sv4);
    assert_eq!(std::net::SocketAddr::from(w5500_sv4), std_s);

    assert_eq!(w5500_ll::net::SocketAddrV4::from(&std_sv4), w5500_sv4);
    assert_eq!(std::net::SocketAddrV4::from(&w5500_sv4), std_sv4);
    assert_eq!(std::net::SocketAddr::from(&w5500_sv4), std_s);
}
