/// Cipher Suites.
///
/// # References
///
/// * [RFC 8446 Appendix B.4](https://datatracker.ietf.org/doc/html/rfc8446#appendix-B.4)
/// * [RFC 8446 Section 9.1](https://datatracker.ietf.org/doc/html/rfc8446#section-9.1)
///
/// +------------------------------+----------------+
/// | Description                  | Value          |
/// +------------------------------+----------------+
/// | TLS_AES_128_GCM_SHA256       | `[0x13, 0x01]` |
/// | TLS_AES_256_GCM_SHA384       | `[0x13, 0x02]` |
/// | TLS_CHACHA20_POLY1305_SHA256 | `[0x13, 0x03]` |
/// | TLS_AES_128_CCM_SHA256       | `[0x13, 0x04]` |
/// | TLS_AES_128_CCM_8_SHA256     | `[0x13, 0x05]` |
/// +------------------------------+----------------+
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[allow(non_camel_case_types)]
pub enum CipherSuite {
    // required
    TLS_AES_128_GCM_SHA256,
    TLS_AES_256_GCM_SHA384,
    TLS_CHACHA20_POLY1305_SHA256,
    TLS_AES_128_CCM_SHA256,
    TLS_AES_128_CCM_8_SHA256,
}

impl CipherSuite {
    pub const fn value(&self) -> [u8; 2] {
        match self {
            Self::TLS_AES_128_GCM_SHA256 => [0x13, 0x01],
            Self::TLS_AES_256_GCM_SHA384 => [0x13, 0x02],
            Self::TLS_CHACHA20_POLY1305_SHA256 => [0x13, 0x03],
            Self::TLS_AES_128_CCM_SHA256 => [0x13, 0x04],
            Self::TLS_AES_128_CCM_8_SHA256 => [0x13, 0x05],
        }
    }
}

impl From<CipherSuite> for [u8; 2] {
    #[inline]
    fn from(cipher_suite: CipherSuite) -> Self {
        cipher_suite.value()
    }
}

impl TryFrom<[u8; 2]> for CipherSuite {
    type Error = [u8; 2];

    fn try_from(value: [u8; 2]) -> Result<Self, Self::Error> {
        match value {
            [0x13, 0x01] => Ok(Self::TLS_AES_128_GCM_SHA256),
            [0x13, 0x02] => Ok(Self::TLS_AES_256_GCM_SHA384),
            [0x13, 0x03] => Ok(Self::TLS_CHACHA20_POLY1305_SHA256),
            [0x13, 0x04] => Ok(Self::TLS_AES_128_CCM_SHA256),
            [0x13, 0x05] => Ok(Self::TLS_AES_128_CCM_8_SHA256),
            _ => Err(value),
        }
    }
}
