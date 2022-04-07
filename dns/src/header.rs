/// Query or Response flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub(crate) enum Qr {
    Query,
    Response,
}

/// DNS response code.
///
/// # References
///
/// * [RTC 1035 Section 4.1.1](https://www.rfc-editor.org/rfc/rfc1035#section-4.1.1)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum ResponseCode {
    /// No Error.
    ///
    /// No error condition.
    NoError = 0,
    /// Format Error.
    ///
    /// The name server was unable to interpret the query.
    FormatError = 1,
    /// Server Failure.
    ///
    /// The name server was unable to process this query due to a problem with
    /// the name server.
    ServerFailure = 2,
    /// Name Error.
    ///
    /// Meaningful only for responses from an authoritative name server,
    /// this code signifies that the domain name referenced in the query does
    /// not exist.
    NameError = 3,
    /// Not Implemented
    ///
    /// The name server does not support the requested kind of query.
    NotImplemented = 4,
    /// Refused.
    ///
    /// The name server refuses to perform the specified operation for policy
    /// reasons.
    /// For example, a name server may not wish to provide the information to
    /// the particular requester, or a name server may not wish to perform
    /// a particular operation (e.g., zone transfer) for particular data.
    Refused = 5,
}

impl TryFrom<u8> for ResponseCode {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            x if x == ResponseCode::NoError as u8 => Ok(ResponseCode::NoError),
            x if x == ResponseCode::FormatError as u8 => Ok(ResponseCode::FormatError),
            x if x == ResponseCode::ServerFailure as u8 => Ok(ResponseCode::ServerFailure),
            x if x == ResponseCode::NameError as u8 => Ok(ResponseCode::NameError),
            x if x == ResponseCode::NotImplemented as u8 => Ok(ResponseCode::NotImplemented),
            x if x == ResponseCode::Refused as u8 => Ok(ResponseCode::Refused),
            x => Err(x),
        }
    }
}

/// DNS query header
///
/// # References
///
/// * [RTC 1035 Section 4.1.1](https://www.rfc-editor.org/rfc/rfc1035#section-4.1.1)
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub(crate) struct Header {
    buf: [u8; Self::LEN_USIZE],
}

impl Header {
    pub(crate) const LEN: u16 = 12;
    pub(crate) const LEN_USIZE: usize = Self::LEN as usize;

    pub(crate) fn new_query(id: u16) -> Self {
        Self {
            buf: Default::default(),
        }
        .set_id(id)
        .set_qr(Qr::Query)
        .set_rd(true)
    }

    #[inline]
    pub(crate) fn new_buf() -> [u8; Self::LEN_USIZE] {
        [0; Self::LEN_USIZE]
    }

    /// Set the 16-bit identifier.
    #[must_use]
    pub(crate) fn id(&self) -> u16 {
        u16::from_be_bytes(self.buf[..2].try_into().unwrap())
    }

    /// Get the 16-bit identifier.
    #[must_use = "set_id returns a modified Header"]
    pub(crate) fn set_id(mut self, id: u16) -> Self {
        self.buf[..2].copy_from_slice(id.to_be_bytes().as_slice());
        self
    }

    #[must_use]
    pub(crate) const fn qr(&self) -> Qr {
        if self.buf[2] & 0x80 == 0x00 {
            Qr::Query
        } else {
            Qr::Response
        }
    }

    #[must_use]
    pub(crate) const fn set_qr(mut self, qr: Qr) -> Self {
        match qr {
            Qr::Query => self.buf[2] &= !0x80,
            Qr::Response => self.buf[2] |= 0x80,
        };
        self
    }

    #[must_use]
    pub(crate) fn set_rd(mut self, rd: bool) -> Self {
        if rd {
            self.buf[2] |= 0x01;
        } else {
            self.buf[2] &= !0x1;
        };
        self
    }

    pub(crate) fn rcode(&self) -> Result<ResponseCode, u8> {
        ResponseCode::try_from(self.buf[3] & 0xF)
    }

    #[must_use]
    pub(crate) fn qdcount(&self) -> u16 {
        u16::from_be_bytes(self.buf[4..6].try_into().unwrap())
    }

    pub(crate) fn set_qdcount(&mut self, qdcount: u16) {
        self.buf[4..6].copy_from_slice(qdcount.to_be_bytes().as_slice());
    }

    pub(crate) fn increment_qdcount(&mut self) {
        let qdcount: u16 = self.qdcount().saturating_add(1);
        self.set_qdcount(qdcount)
    }

    #[must_use]
    pub(crate) fn ancount(&self) -> u16 {
        u16::from_be_bytes(self.buf[6..8].try_into().unwrap())
    }

    #[must_use]
    #[allow(dead_code)]
    pub(crate) fn arcount(&self) -> u16 {
        u16::from_be_bytes(self.buf[10..12].try_into().unwrap())
    }

    #[must_use]
    pub(crate) fn as_bytes(&self) -> &[u8; Self::LEN_USIZE] {
        &self.buf
    }
}

impl From<[u8; Header::LEN_USIZE]> for Header {
    #[inline]
    fn from(buf: [u8; Self::LEN_USIZE]) -> Self {
        Header { buf }
    }
}

#[cfg(test)]
mod tests {
    use super::Header;

    #[test]
    fn qdcount() {
        let mut header: Header = Header { buf: [0; 12] };

        header.set_qdcount(0xABCD);
        assert_eq!(header.qdcount(), 0xABCD);

        header.increment_qdcount();
        assert_eq!(header.qdcount(), 0xABCE);
    }
}
