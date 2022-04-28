use crate::{AlertDescription, TlsVersion};

/// Content Type.
///
/// # References
///
/// * [RFC 8446 Section 5.1](https://datatracker.ietf.org/doc/html/rfc8446#section-5.1)
///
/// ```text
/// enum {
///     invalid(0),
///     change_cipher_spec(20),
///     alert(21),
///     handshake(22),
///     application_data(23),
///     (255)
/// } ContentType;
/// ```
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ContentType {
    ChangeCipherSpec = 20,
    Alert = 21,
    Handshake = 22,
    ApplicationData = 23,
}

impl From<ContentType> for u8 {
    #[inline]
    fn from(content_type: ContentType) -> Self {
        content_type as u8
    }
}

impl TryFrom<u8> for ContentType {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            x if x == (Self::ChangeCipherSpec as u8) => Ok(Self::ChangeCipherSpec),
            x if x == (Self::Alert as u8) => Ok(Self::Alert),
            x if x == (Self::Handshake as u8) => Ok(Self::Handshake),
            x if x == (Self::ApplicationData as u8) => Ok(Self::ApplicationData),
            _ => Err(value),
        }
    }
}

/// # References
///
/// * [RFC 8446 Appendix B.1](https://datatracker.ietf.org/doc/html/rfc8446#appendix-B.1)
///
/// ```text
/// struct {
///     ContentType type;
///     ProtocolVersion legacy_record_version;
///     uint16 length;
///     opaque fragment[TLSPlaintext.length];
/// } TLSPlaintext;
/// ```
pub struct RecordHeader {
    buf: [u8; Self::LEN],
}

impl RecordHeader {
    pub const LEN: usize = 5;

    pub fn content_type(&self) -> ContentType {
        // validated in constructor - will never panic
        ContentType::try_from(self.buf[0]).unwrap()
    }

    pub fn length(&self) -> u16 {
        // unwrap should get optimized away
        u16::from_be_bytes(self.buf[3..5].try_into().unwrap())
    }

    pub fn as_bytes(&self) -> &[u8; 5] {
        &self.buf
    }

    pub fn ser(content_type: ContentType, data_len: u16) -> Self {
        RecordHeader {
            buf: [
                content_type.into(),
                TlsVersion::V1_2.msb(),
                TlsVersion::V1_2.lsb(),
                (data_len >> 8) as u8,
                data_len as u8,
            ],
        }
    }

    pub fn deser(buf: [u8; 5]) -> Result<Self, AlertDescription> {
        match ContentType::try_from(buf[0]) {
            Ok(content_type) => content_type,
            Err(content_type) => {
                error!("invalid ContentType value: 0x{:02X}", content_type);
                return Err(AlertDescription::IllegalParameter);
            }
        };

        // https://www.rfc-editor.org/rfc/rfc8446#appendix-D
        // The value of TLSPlaintext.legacy_record_version MUST be ignored by all
        // implementations.  The value of TLSCiphertext.legacy_record_version is
        // included in the additional data for deprotection but MAY otherwise be
        // ignored or MAY be validated to match the fixed constant value.

        Ok(Self { buf })
    }
}
