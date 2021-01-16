//! Networking data types.
//!
//! There may a standard for embedded networking in the future, see
//! [rust-embedded issue 348] and [RFC 2832]
//!
//! This is mostly ripped directly from [std::net].
//!
//! [rust-embedded issue 348]: https://github.com/rust-embedded/wg/issues/348
//! [std::net]: https://doc.rust-lang.org/std/net/index.html
//! [RFC 2832]: https://github.com/rust-lang/rfcs/pull/2832

/// Ipv4Addr address struct.
///
/// Can be instantiated with `Ipv4Addr::new`.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Hash, Default)]
pub struct Ipv4Addr {
    /// Octets of the Ipv4Addr address.
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
    /// String formatter for Ipv4Addr addresses.
    fn fmt(&self, fmt: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        write!(
            fmt,
            "{}.{}.{}.{}",
            self.octets[0], self.octets[1], self.octets[2], self.octets[3],
        )
    }
}

/// EUI-48 MAC address struct.
///
/// Can be instantiated with `Eui48Addr::new`.
///
/// This is an EUI-48 MAC address (previously called MAC-48).
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
    /// String formatter for Eui48Addr addresses.
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
