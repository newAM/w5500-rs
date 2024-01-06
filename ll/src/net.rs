//! Networking data types.

pub use core::net::{Ipv4Addr, SocketAddrV4};

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
