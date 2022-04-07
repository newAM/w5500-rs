/// The class of a DNS query.
///
/// # References
///
/// * [RFC 1035 Section 3.2.4](https://tools.ietf.org/rfc/rfc1035#section-3.2.4).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u16)]
pub enum Qclass {
    /// Internet
    IN = 1,
    /// CSNET (obsolete)
    CS = 2,
    /// CHAOS
    CH = 3,
    /// Hesiod
    HS = 4,
    /// Any class
    ANY = 5,
}

impl Qclass {
    pub(crate) const fn high_byte(&self) -> u8 {
        // I hope the compiler can figure out this is always zero
        ((*self as u16) >> 8) as u8
    }

    pub(crate) const fn low_byte(&self) -> u8 {
        *self as u8
    }
}

impl TryFrom<u16> for Qclass {
    type Error = u16;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            x if x == (Self::IN as u16) => Ok(Self::IN),
            x if x == (Self::CS as u16) => Ok(Self::CS),
            x if x == (Self::CH as u16) => Ok(Self::CH),
            x if x == (Self::HS as u16) => Ok(Self::HS),
            x if x == (Self::ANY as u16) => Ok(Self::ANY),
            x => Err(x),
        }
    }
}
