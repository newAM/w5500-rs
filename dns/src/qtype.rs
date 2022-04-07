/// DNS question query type.
///
/// # References
///
/// * [RFC 1035 Section 3.2.2](https://tools.ietf.org/rfc/rfc1035#section-3.2.2)
/// * [RFC 1035 Section 3.2.3](https://tools.ietf.org/rfc/rfc1035#section-3.2.3)
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u16)]
pub enum Qtype {
    /// a host address
    A = 1,
    /// an authoritative name server
    NS = 2,
    /// a mail destination (Obsolete - use MX)
    MD = 3,
    /// a mail forwarder (Obsolete - use MX)
    MF = 4,
    /// the canonical name for an alias
    CNAME = 5,
    /// marks the start of a zone of authority
    SOA = 6,
    /// a mailbox domain name (EXPERIMENTAL)
    MB = 7,
    /// a mail group member (EXPERIMENTAL)
    MG = 8,
    /// a mail rename domain name (EXPERIMENTAL)
    MR = 9,
    /// a null RR (EXPERIMENTAL)
    NULL = 10,
    /// a well known service description
    WKS = 11,
    /// a domain name pointer
    PTR = 12,
    /// host information
    HINFO = 13,
    /// mailbox or mail list information
    MINFO = 14,
    /// mail exchange
    MX = 15,
    /// text strings
    TXT = 16,
    /// A request for a transfer of an entire zone
    AXFR = 252,
    /// A request for mailbox-related records (MB, MG or MR)
    MAILB = 253,
    /// A request for mail agent RRs (Obsolete - see MX)
    MAILA = 254,
    /// A request for all records
    ALL = 255,
}

impl Qtype {
    pub(crate) const fn high_byte(&self) -> u8 {
        // I hope the compiler can figure out this is always zero
        ((*self as u16) >> 8) as u8
    }

    pub(crate) const fn low_byte(&self) -> u8 {
        *self as u8
    }
}

impl TryFrom<u16> for Qtype {
    type Error = u16;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            x if x == (Self::A as u16) => Ok(Self::A),
            x if x == (Self::NS as u16) => Ok(Self::NS),
            x if x == (Self::MD as u16) => Ok(Self::MD),
            x if x == (Self::MF as u16) => Ok(Self::MF),
            x if x == (Self::CNAME as u16) => Ok(Self::CNAME),
            x if x == (Self::SOA as u16) => Ok(Self::SOA),
            x if x == (Self::MB as u16) => Ok(Self::MB),
            x if x == (Self::MG as u16) => Ok(Self::MG),
            x if x == (Self::MR as u16) => Ok(Self::MR),
            x if x == (Self::NULL as u16) => Ok(Self::NULL),
            x if x == (Self::WKS as u16) => Ok(Self::WKS),
            x if x == (Self::PTR as u16) => Ok(Self::PTR),
            x if x == (Self::HINFO as u16) => Ok(Self::HINFO),
            x if x == (Self::MINFO as u16) => Ok(Self::MINFO),
            x if x == (Self::MX as u16) => Ok(Self::MX),
            x if x == (Self::TXT as u16) => Ok(Self::TXT),
            x if x == (Self::AXFR as u16) => Ok(Self::AXFR),
            x if x == (Self::MAILB as u16) => Ok(Self::MAILB),
            x if x == (Self::MAILA as u16) => Ok(Self::MAILA),
            x if x == (Self::ALL as u16) => Ok(Self::ALL),
            x => Err(x),
        }
    }
}
