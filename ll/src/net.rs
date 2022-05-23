//! Networking data types.
//!
//! There may a standard for embedded networking in the future, see
//! [rust-embedded issue 348] and [RFC 2832]
//!
//! This is mostly ripped from [`std::net`].
//!
//! [rust-embedded issue 348]: https://github.com/rust-embedded/wg/issues/348
//! [`std::net`]: https://doc.rust-lang.org/std/net/index.html
//! [RFC 2832]: https://github.com/rust-lang/rfcs/pull/2832

/// Ipv4Addr address struct.
///
/// Can be instantiated with [`Ipv4Addr::new`].
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Hash, Default)]
pub struct Ipv4Addr {
    /// Octets of the [`Ipv4Addr`] address.
    pub octets: [u8; 4],
}

impl Ipv4Addr {
    /// Creates a new IPv4 address from four eight-bit octets.
    ///
    /// The result will represent the IP address `a`.`b`.`c`.`d`.
    ///
    /// # Examples
    ///
    /// ```
    /// use w5500_ll::net::Ipv4Addr;
    ///
    /// let addr = Ipv4Addr::new(127, 0, 0, 1);
    /// ```
    #[allow(clippy::many_single_char_names)]
    pub const fn new(a: u8, b: u8, c: u8, d: u8) -> Ipv4Addr {
        Ipv4Addr {
            octets: [a, b, c, d],
        }
    }

    /// An IPv4 address with the address pointing to localhost: 127.0.0.1.
    ///
    /// # Examples
    ///
    /// ```
    /// use w5500_ll::net::Ipv4Addr;
    ///
    /// let addr = Ipv4Addr::LOCALHOST;
    /// assert_eq!(addr, Ipv4Addr::new(127, 0, 0, 1));
    /// ```
    pub const LOCALHOST: Self = Ipv4Addr::new(127, 0, 0, 1);

    /// An IPv4 address representing an unspecified address: 0.0.0.0
    ///
    /// # Examples
    ///
    /// ```
    /// use w5500_ll::net::Ipv4Addr;
    ///
    /// let addr = Ipv4Addr::UNSPECIFIED;
    /// assert_eq!(addr, Ipv4Addr::new(0, 0, 0, 0));
    /// ```
    pub const UNSPECIFIED: Self = Ipv4Addr::new(0, 0, 0, 0);

    /// An IPv4 address representing the broadcast address: 255.255.255.255
    ///
    /// # Examples
    ///
    /// ```
    /// use w5500_ll::net::Ipv4Addr;
    ///
    /// let addr = Ipv4Addr::BROADCAST;
    /// assert_eq!(addr, Ipv4Addr::new(255, 255, 255, 255));
    /// ```
    pub const BROADCAST: Self = Ipv4Addr::new(255, 255, 255, 255);

    /// Returns [`true`] for the special 'unspecified' address (0.0.0.0).
    ///
    /// This property is defined in _UNIX Network Programming, Second Edition_,
    /// W. Richard Stevens, p. 891; see also [ip7].
    ///
    /// [ip7]: http://man7.org/linux/man-pages/man7/ip.7.html
    ///
    /// # Examples
    ///
    /// ```
    /// use w5500_ll::net::Ipv4Addr;
    ///
    /// assert_eq!(Ipv4Addr::new(0, 0, 0, 0).is_unspecified(), true);
    /// assert_eq!(Ipv4Addr::new(45, 22, 13, 197).is_unspecified(), false);
    /// ```
    pub const fn is_unspecified(&self) -> bool {
        self.octets[0] == 0 && self.octets[1] == 0 && self.octets[2] == 0 && self.octets[3] == 0
    }

    /// Returns [`true`] if this is a loopback address (127.0.0.0/8).
    ///
    /// This property is defined by [IETF RFC 1122].
    ///
    /// [IETF RFC 1122]: https://tools.ietf.org/html/rfc1122
    ///
    /// # Examples
    ///
    /// ```
    /// use w5500_ll::net::Ipv4Addr;
    ///
    /// assert_eq!(Ipv4Addr::new(127, 0, 0, 1).is_loopback(), true);
    /// assert_eq!(Ipv4Addr::new(45, 22, 13, 197).is_loopback(), false);
    /// ```
    pub const fn is_loopback(&self) -> bool {
        self.octets[0] == 127
    }

    /// Returns [`true`] if this is a private address.
    ///
    /// The private address ranges are defined in [IETF RFC 1918] and include:
    ///
    ///  - 10.0.0.0/8
    ///  - 172.16.0.0/12
    ///  - 192.168.0.0/16
    ///
    /// [IETF RFC 1918]: https://tools.ietf.org/html/rfc1918
    ///
    /// # Examples
    ///
    /// ```
    /// use w5500_ll::net::Ipv4Addr;
    ///
    /// assert_eq!(Ipv4Addr::new(10, 0, 0, 1).is_private(), true);
    /// assert_eq!(Ipv4Addr::new(10, 10, 10, 10).is_private(), true);
    /// assert_eq!(Ipv4Addr::new(172, 16, 10, 10).is_private(), true);
    /// assert_eq!(Ipv4Addr::new(172, 29, 45, 14).is_private(), true);
    /// assert_eq!(Ipv4Addr::new(172, 32, 0, 2).is_private(), false);
    /// assert_eq!(Ipv4Addr::new(192, 168, 0, 2).is_private(), true);
    /// assert_eq!(Ipv4Addr::new(192, 169, 0, 2).is_private(), false);
    /// ```
    #[allow(clippy::manual_range_contains)]
    pub const fn is_private(&self) -> bool {
        match self.octets {
            [10, ..] => true,
            [172, b, ..] if b >= 16 && b <= 31 => true,
            [192, 168, ..] => true,
            _ => false,
        }
    }

    /// Returns [`true`] if the address is link-local (169.254.0.0/16).
    ///
    /// This property is defined by [IETF RFC 3927].
    ///
    /// [IETF RFC 3927]: https://tools.ietf.org/html/rfc3927
    ///
    /// # Examples
    ///
    /// ```
    /// use w5500_ll::net::Ipv4Addr;
    ///
    /// assert_eq!(Ipv4Addr::new(169, 254, 0, 0).is_link_local(), true);
    /// assert_eq!(Ipv4Addr::new(169, 254, 10, 65).is_link_local(), true);
    /// assert_eq!(Ipv4Addr::new(16, 89, 10, 65).is_link_local(), false);
    /// ```
    pub const fn is_link_local(&self) -> bool {
        matches!(self.octets, [169, 254, ..])
    }

    /// Returns [`true`] if this is a multicast address (224.0.0.0/4).
    ///
    /// Multicast addresses have a most significant octet between 224 and 239,
    /// and is defined by [IETF RFC 5771].
    ///
    /// [IETF RFC 5771]: https://tools.ietf.org/html/rfc5771
    ///
    /// # Examples
    ///
    /// ```
    /// use w5500_ll::net::Ipv4Addr;
    ///
    /// assert_eq!(Ipv4Addr::new(224, 254, 0, 0).is_multicast(), true);
    /// assert_eq!(Ipv4Addr::new(236, 168, 10, 65).is_multicast(), true);
    /// assert_eq!(Ipv4Addr::new(172, 16, 10, 65).is_multicast(), false);
    /// ```
    pub const fn is_multicast(&self) -> bool {
        self.octets[0] >= 224 && self.octets[0] <= 239
    }

    /// Returns [`true`] if this is a broadcast address (255.255.255.255).
    ///
    /// A broadcast address has all octets set to 255 as defined in [IETF RFC 919].
    ///
    /// [IETF RFC 919]: https://tools.ietf.org/html/rfc919
    ///
    /// # Examples
    ///
    /// ```
    /// use w5500_ll::net::Ipv4Addr;
    ///
    /// assert_eq!(Ipv4Addr::new(255, 255, 255, 255).is_broadcast(), true);
    /// assert_eq!(Ipv4Addr::new(236, 168, 10, 65).is_broadcast(), false);
    /// ```
    pub const fn is_broadcast(&self) -> bool {
        u32::from_be_bytes(self.octets) == u32::from_be_bytes(Self::BROADCAST.octets)
    }

    /// Returns [`true`] if this address is in a range designated for documentation.
    ///
    /// This is defined in [IETF RFC 5737]:
    ///
    /// - 192.0.2.0/24 (TEST-NET-1)
    /// - 198.51.100.0/24 (TEST-NET-2)
    /// - 203.0.113.0/24 (TEST-NET-3)
    ///
    /// [IETF RFC 5737]: https://tools.ietf.org/html/rfc5737
    ///
    /// # Examples
    ///
    /// ```
    /// use w5500_ll::net::Ipv4Addr;
    ///
    /// assert_eq!(Ipv4Addr::new(192, 0, 2, 255).is_documentation(), true);
    /// assert_eq!(Ipv4Addr::new(198, 51, 100, 65).is_documentation(), true);
    /// assert_eq!(Ipv4Addr::new(203, 0, 113, 6).is_documentation(), true);
    /// assert_eq!(Ipv4Addr::new(193, 34, 17, 19).is_documentation(), false);
    /// ```
    #[allow(clippy::match_like_matches_macro)]
    pub const fn is_documentation(&self) -> bool {
        match self.octets {
            [192, 0, 2, _] => true,
            [198, 51, 100, _] => true,
            [203, 0, 113, _] => true,
            _ => false,
        }
    }
}

impl ::core::fmt::Display for Ipv4Addr {
    /// String formatter for [`Ipv4Addr`] addresses.
    fn fmt(&self, fmt: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        write!(
            fmt,
            "{}.{}.{}.{}",
            self.octets[0], self.octets[1], self.octets[2], self.octets[3],
        )
    }
}

impl From<[u8; 4]> for Ipv4Addr {
    /// Creates an `Ipv4Addr` from a four element byte array.
    ///
    /// # Examples
    ///
    /// ```
    /// use w5500_ll::net::Ipv4Addr;
    ///
    /// let addr = Ipv4Addr::from([13u8, 12u8, 11u8, 10u8]);
    /// assert_eq!(Ipv4Addr::new(13, 12, 11, 10), addr);
    /// ```
    fn from(octets: [u8; 4]) -> Self {
        Ipv4Addr { octets }
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for Ipv4Addr {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "{}.{}.{}.{}",
            self.octets[0],
            self.octets[1],
            self.octets[2],
            self.octets[3]
        )
    }
}

#[cfg(feature = "ufmt")]
impl ufmt::uDisplay for Ipv4Addr {
    fn fmt<W>(&self, f: &mut ufmt::Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: ufmt::uWrite + ?Sized,
    {
        ufmt::uwrite!(
            f,
            "{}.{}.{}.{}",
            self.octets[0],
            self.octets[1],
            self.octets[2],
            self.octets[3]
        )
    }
}

#[cfg(feature = "std")]
impl From<std::net::Ipv4Addr> for Ipv4Addr {
    fn from(ip: std::net::Ipv4Addr) -> Self {
        Ipv4Addr::new(
            ip.octets()[0],
            ip.octets()[1],
            ip.octets()[2],
            ip.octets()[3],
        )
    }
}

#[cfg(feature = "std")]
impl From<&std::net::Ipv4Addr> for Ipv4Addr {
    fn from(ip: &std::net::Ipv4Addr) -> Self {
        Ipv4Addr::new(
            ip.octets()[0],
            ip.octets()[1],
            ip.octets()[2],
            ip.octets()[3],
        )
    }
}

#[cfg(feature = "std")]
impl From<Ipv4Addr> for std::net::Ipv4Addr {
    fn from(ip: Ipv4Addr) -> Self {
        std::net::Ipv4Addr::new(ip.octets[0], ip.octets[1], ip.octets[2], ip.octets[3])
    }
}

#[cfg(feature = "std")]
impl From<&Ipv4Addr> for std::net::Ipv4Addr {
    fn from(ip: &Ipv4Addr) -> Self {
        std::net::Ipv4Addr::new(ip.octets[0], ip.octets[1], ip.octets[2], ip.octets[3])
    }
}

#[cfg(feature = "std")]
impl From<Ipv4Addr> for std::net::IpAddr {
    fn from(ip: Ipv4Addr) -> Self {
        std::net::IpAddr::V4(ip.into())
    }
}

#[cfg(feature = "std")]
impl From<&Ipv4Addr> for std::net::IpAddr {
    fn from(ip: &Ipv4Addr) -> Self {
        std::net::IpAddr::V4(ip.into())
    }
}

/// EUI-48 MAC address struct.
///
/// Can be instantiated with [`Eui48Addr::new`].
///
/// This is an EUI-48 [MAC address] (previously called MAC-48).
///
/// [MAC address]: https://en.wikipedia.org/wiki/MAC_address
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Hash, Default)]
pub struct Eui48Addr {
    /// Octets of the MAC address.
    pub octets: [u8; 6],
}

impl Eui48Addr {
    /// Creates a new EUI-48 MAC address from six eight-bit octets.
    ///
    /// The result will represent the EUI-48 MAC address
    /// `a`:`b`:`c`:`d`:`e`:`f`.
    ///
    /// # Examples
    ///
    /// ```
    /// use w5500_ll::net::Eui48Addr;
    ///
    /// let addr = Eui48Addr::new(0x00, 0x00, 0x5E, 0x00, 0x00, 0x00);
    /// ```
    #[allow(clippy::many_single_char_names)]
    pub const fn new(a: u8, b: u8, c: u8, d: u8, e: u8, f: u8) -> Eui48Addr {
        Eui48Addr {
            octets: [a, b, c, d, e, f],
        }
    }

    /// An EUI-48 MAC address representing an unspecified address:
    /// 00:00:00:00:00:00
    ///
    /// # Examples
    ///
    /// ```
    /// use w5500_ll::net::Eui48Addr;
    ///
    /// let addr = Eui48Addr::UNSPECIFIED;
    /// assert_eq!(addr, Eui48Addr::new(0x00, 0x00, 0x00, 0x00, 0x00, 0x00));
    /// ```
    pub const UNSPECIFIED: Self = Eui48Addr::new(0, 0, 0, 0, 0, 0);
}

impl ::core::fmt::Display for Eui48Addr {
    /// String formatter for [`Eui48Addr`] addresses.
    fn fmt(&self, fmt: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        write!(
            fmt,
            "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            self.octets[0],
            self.octets[1],
            self.octets[2],
            self.octets[3],
            self.octets[4],
            self.octets[5],
        )
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for Eui48Addr {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            self.octets[0],
            self.octets[1],
            self.octets[2],
            self.octets[3],
            self.octets[4],
            self.octets[5],
        )
    }
}

#[cfg(feature = "ufmt")]
impl ufmt::uDisplay for Eui48Addr {
    fn fmt<W>(&self, f: &mut ufmt::Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: ufmt::uWrite + ?Sized,
    {
        // https://github.com/japaric/ufmt/pull/35
        ufmt::uwrite!(
            f,
            "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            self.octets[0],
            self.octets[1],
            self.octets[2],
            self.octets[3],
            self.octets[4],
            self.octets[5],
        )
    }
}

impl From<[u8; 6]> for Eui48Addr {
    /// Creates an `Eui48Addr` from a six element byte array.
    ///
    /// # Examples
    ///
    /// ```
    /// use w5500_ll::net::Eui48Addr;
    ///
    /// let addr = Eui48Addr::from([0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC]);
    /// assert_eq!(Eui48Addr::new(0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC), addr);
    /// ```
    fn from(octets: [u8; 6]) -> Self {
        Eui48Addr { octets }
    }
}

/// An IPv4 socket address.
///
/// This is mostly ripped from [`std::net::SocketAddrV4`].
///
/// IPv4 socket addresses consist of an [`IPv4` address] and a 16-bit port
/// number, as stated in [IETF RFC 793].
///
/// Can be instantiated with [`SocketAddrV4::new`].
///
/// # Examples
///
/// ```
/// use w5500_ll::net::{Ipv4Addr, SocketAddrV4};
///
/// let socket = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080);
///
/// assert_eq!(socket.ip(), &Ipv4Addr::new(127, 0, 0, 1));
/// assert_eq!(socket.port(), 8080);
/// ```
///
/// [IETF RFC 793]: https://tools.ietf.org/html/rfc793
/// [`IPv4` address]: Ipv4Addr
/// [`std::net::SocketAddrV4`]: https://doc.rust-lang.org/std/net/struct.SocketAddrV4.html
#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Hash, Default)]
pub struct SocketAddrV4 {
    ip: Ipv4Addr,
    port: u16,
}

impl SocketAddrV4 {
    /// Creates a new socket address from an [`IPv4` address] and a port number.
    ///
    /// [`IPv4` address]: Ipv4Addr
    ///
    /// # Examples
    ///
    /// ```
    /// use w5500_ll::net::{Ipv4Addr, SocketAddrV4};
    ///
    /// let addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080);
    /// ```
    pub const fn new(ip: Ipv4Addr, port: u16) -> SocketAddrV4 {
        SocketAddrV4 { ip, port }
    }

    /// Returns the IP address associated with this socket address.
    ///
    /// # Examples
    ///
    /// ```
    /// use w5500_ll::net::{Ipv4Addr, SocketAddrV4};
    ///
    /// let addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080);
    /// assert_eq!(addr.ip(), &Ipv4Addr::new(127, 0, 0, 1));
    /// ```
    pub const fn ip(&self) -> &Ipv4Addr {
        &self.ip
    }

    /// Changes the IP address associated with this socket address.
    ///
    /// # Examples
    ///
    /// ```
    /// use w5500_ll::net::{Ipv4Addr, SocketAddrV4};
    ///
    /// let mut addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080);
    /// addr.set_ip(Ipv4Addr::new(192, 168, 0, 1));
    /// assert_eq!(addr.ip(), &Ipv4Addr::new(192, 168, 0, 1));
    /// ```
    pub fn set_ip(&mut self, new_ip: Ipv4Addr) {
        self.ip = new_ip
    }

    /// Returns the port number associated with this socket address.
    ///
    /// # Examples
    ///
    /// ```
    /// use w5500_ll::net::{Ipv4Addr, SocketAddrV4};
    ///
    /// let addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080);
    /// assert_eq!(addr.port(), 8080);
    /// ```
    pub const fn port(&self) -> u16 {
        self.port
    }

    /// Changes the port number associated with this socket address.
    ///
    /// # Examples
    ///
    /// ```
    /// use w5500_ll::net::{Ipv4Addr, SocketAddrV4};
    ///
    /// let mut addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080);
    /// addr.set_port(4242);
    /// assert_eq!(addr.port(), 4242);
    /// ```
    pub fn set_port(&mut self, new_port: u16) {
        self.port = new_port
    }
}

impl ::core::fmt::Display for SocketAddrV4 {
    /// String formatter for [`SocketAddrV4`] addresses.
    fn fmt(&self, fmt: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        write!(fmt, "{}:{}", self.ip(), self.port())
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for SocketAddrV4 {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "{:?}:{}", self.ip(), self.port())
    }
}


#[cfg(feature = "ufmt")]
impl ufmt::uDisplay for SocketAddrV4 {
    fn fmt<W>(&self, f: &mut ufmt::Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: ufmt::uWrite + ?Sized,
    {
        ufmt::uwrite!(f, "{}:{}", self.ip(), self.port())
    }
}

#[cfg(feature = "std")]
impl From<std::net::SocketAddrV4> for SocketAddrV4 {
    fn from(addr: std::net::SocketAddrV4) -> Self {
        SocketAddrV4::new(addr.ip().into(), addr.port())
    }
}

#[cfg(feature = "std")]
impl From<&std::net::SocketAddrV4> for SocketAddrV4 {
    fn from(addr: &std::net::SocketAddrV4) -> Self {
        SocketAddrV4::new(addr.ip().into(), addr.port())
    }
}

#[cfg(feature = "std")]
impl From<SocketAddrV4> for std::net::SocketAddrV4 {
    fn from(addr: SocketAddrV4) -> Self {
        std::net::SocketAddrV4::new(addr.ip().into(), addr.port)
    }
}

#[cfg(feature = "std")]
impl From<&SocketAddrV4> for std::net::SocketAddrV4 {
    fn from(addr: &SocketAddrV4) -> Self {
        std::net::SocketAddrV4::new(addr.ip().into(), addr.port)
    }
}

#[cfg(feature = "std")]
impl From<&SocketAddrV4> for std::net::SocketAddr {
    fn from(addr: &SocketAddrV4) -> Self {
        std::net::SocketAddr::V4(addr.into())
    }
}

#[cfg(feature = "std")]
impl From<SocketAddrV4> for std::net::SocketAddr {
    fn from(addr: SocketAddrV4) -> Self {
        std::net::SocketAddr::V4(addr.into())
    }
}
