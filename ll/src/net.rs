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
