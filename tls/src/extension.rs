/// Extensions.
///
/// # References
///
/// * [RFC 8446 Section 4.2](https://datatracker.ietf.org/doc/html/rfc8446#section-4.2)
/// * [RFC 8449](https://datatracker.ietf.org/doc/html/rfc8449)
#[repr(u16)]
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "ufmt", derive(ufmt::derive::uDebug))]
pub enum ExtensionType {
    ServerName = 0,                           // RFC 6066
    MaxFragmentLength = 1,                    // RFC 6066
    StatusRequest = 5,                        // RFC 6066
    SupportedGroups = 10,                     // RFC 8422, 7919
    SignatureAlgorithms = 13,                 // RFC 8446
    UseSrtp = 14,                             // RFC 5764
    Heartbeat = 15,                           // RFC 6520
    ApplicationLayerProtocolNegotiation = 16, // RFC 7301
    SignedCertificateTimestamp = 18,          // RFC 6962
    ClientCertificateType = 19,               // RFC 7250
    ServerCertificateType = 20,               // RFC 7250
    Padding = 21,                             // RFC 7685
    RecordSizeLimit = 28,                     // RFC 8449
    PreSharedKey = 41,                        // RFC 8446
    EarlyData = 42,                           // RFC 8446
    SupportedVersions = 43,                   // RFC 8446
    Cookie = 44,                              // RFC 8446
    PskKeyExchangeModes = 45,                 // RFC 8446
    CertificateAuthorities = 47,              // RFC 8446
    OidFilters = 48,                          // RFC 8446
    PostHandshakeAuth = 49,                   // RFC 8446
    SignatureAlgorithmsCert = 50,             // RFC 8446
    KeyShare = 51,                            // RFC 8446
}

impl ExtensionType {
    pub const fn msb(self) -> u8 {
        ((self as u16) >> 8) as u8
    }

    pub const fn lsb(self) -> u8 {
        self as u8
    }
}

impl From<ExtensionType> for u16 {
    #[inline]
    fn from(extension_type: ExtensionType) -> Self {
        extension_type as u16
    }
}

impl TryFrom<u16> for ExtensionType {
    type Error = u16;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            x if x == (Self::ServerName as u16) => Ok(Self::ServerName),
            x if x == (Self::MaxFragmentLength as u16) => Ok(Self::MaxFragmentLength),
            x if x == (Self::StatusRequest as u16) => Ok(Self::StatusRequest),
            x if x == (Self::SupportedGroups as u16) => Ok(Self::SupportedGroups),
            x if x == (Self::SignatureAlgorithms as u16) => Ok(Self::SignatureAlgorithms),
            x if x == (Self::UseSrtp as u16) => Ok(Self::UseSrtp),
            x if x == (Self::Heartbeat as u16) => Ok(Self::Heartbeat),
            x if x == (Self::ApplicationLayerProtocolNegotiation as u16) => {
                Ok(Self::ApplicationLayerProtocolNegotiation)
            }
            x if x == (Self::SignedCertificateTimestamp as u16) => {
                Ok(Self::SignedCertificateTimestamp)
            }
            x if x == (Self::ClientCertificateType as u16) => Ok(Self::ClientCertificateType),
            x if x == (Self::ServerCertificateType as u16) => Ok(Self::ServerCertificateType),
            x if x == (Self::Padding as u16) => Ok(Self::Padding),
            x if x == (Self::RecordSizeLimit as u16) => Ok(Self::RecordSizeLimit),
            x if x == (Self::PreSharedKey as u16) => Ok(Self::PreSharedKey),
            x if x == (Self::EarlyData as u16) => Ok(Self::EarlyData),
            x if x == (Self::SupportedVersions as u16) => Ok(Self::SupportedVersions),
            x if x == (Self::Cookie as u16) => Ok(Self::Cookie),
            x if x == (Self::PskKeyExchangeModes as u16) => Ok(Self::PskKeyExchangeModes),
            x if x == (Self::CertificateAuthorities as u16) => Ok(Self::CertificateAuthorities),
            x if x == (Self::OidFilters as u16) => Ok(Self::OidFilters),
            x if x == (Self::PostHandshakeAuth as u16) => Ok(Self::PostHandshakeAuth),
            x if x == (Self::SignatureAlgorithmsCert as u16) => Ok(Self::SignatureAlgorithmsCert),
            x if x == (Self::KeyShare as u16) => Ok(Self::KeyShare),
            _ => Err(value),
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "ufmt", derive(ufmt::derive::uDebug))]
pub struct Extension<'a> {
    pub tipe: ExtensionType,
    pub data: &'a [u8],
}
