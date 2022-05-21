// This module contains ugly code because I did a lot in `const` functions,
// which limits what functions I can use.

use hmac::digest::generic_array::{typenum::U32, GenericArray};
use w5500_hl::Hostname;

use crate::{
    cipher_suites::CipherSuite, extension::ExtensionType, key_schedule::KeySchedule, ContentType,
    TlsVersion,
};
use core::mem::size_of;
use sha2::Sha256;

use super::HandshakeType;

macro_rules! const_concat_bytes {
    ($a:expr, $b:expr $(,)*) => {{
        const __LEN: usize = $a.len() + $b.len();
        const __CONCATENATED: [u8; __LEN] = {
            let mut out: [u8; __LEN] = [0u8; __LEN];
            let mut i = 0;
            while i < $a.len() {
                out[i] = $a[i];
                i += 1;
            }
            i = 0;
            while i < $b.len() {
                out[i + $a.len()] = $b[i];
                i += 1;
            }
            out
        };

        __CONCATENATED
    }};
}

/// # References
///
/// * [RFC 8846 Section 4.2.3](https://datatracker.ietf.org/doc/html/rfc8446#section-4.2.3)
/// * [RFC 8446 Section 9.1](https://datatracker.ietf.org/doc/html/rfc8446#section-9.1)
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SignatureScheme {
    // RSASSA-PKCS1-v1_5 algorithms
    RsaPkcs1Sha256 = 0x0401, // required
    RsaPkcs1Sha384 = 0x0501,
    RsaPkcs1Sha512 = 0x0601,
    // ECDSA algorithms
    EcdsaSecp256r1Sha256 = 0x0403, // required
    EcdsaSecp384r1Sha384 = 0x0503,
    EcdsaSecp521r1Sha512 = 0x0603,
    // RSASSA-PSS algorithms with public key OID rsaEncryption
    RsaPssRsaeSha256 = 0x0804, // required
    RsaPssRsaeSha384 = 0x0805,
    RsaPssRsaeSha512 = 0x0806,
    // EdDSA algorithms
    Ed25519 = 0x0807,
    Ed448 = 0x0808,
    // RSASSA-PSS algorithms with public key OID RSASSA-PSS
    RsaPssPssSha256 = 0x0809,
    RsaPssPssSha384 = 0x080a,
    RsaPssPssSha512 = 0x080b,
    // Legacy algorithms
    RsaPkcs1Sha1 = 0x0201,
    EcdsaSha1 = 0x0203,
    // private_use(0xFE00..0xFFFF),
    // (0xFFFF)
}

impl From<SignatureScheme> for u16 {
    #[inline]
    fn from(signature_scheme: SignatureScheme) -> Self {
        signature_scheme as u16
    }
}

impl TryFrom<u16> for SignatureScheme {
    type Error = u16;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            x if x == (Self::RsaPkcs1Sha256 as u16) => Ok(Self::RsaPkcs1Sha256),
            x if x == (Self::RsaPkcs1Sha384 as u16) => Ok(Self::RsaPkcs1Sha384),
            x if x == (Self::RsaPkcs1Sha512 as u16) => Ok(Self::RsaPkcs1Sha512),
            x if x == (Self::EcdsaSecp256r1Sha256 as u16) => Ok(Self::EcdsaSecp256r1Sha256),
            x if x == (Self::EcdsaSecp384r1Sha384 as u16) => Ok(Self::EcdsaSecp384r1Sha384),
            x if x == (Self::EcdsaSecp521r1Sha512 as u16) => Ok(Self::EcdsaSecp521r1Sha512),
            x if x == (Self::RsaPssRsaeSha256 as u16) => Ok(Self::RsaPssRsaeSha256),
            x if x == (Self::RsaPssRsaeSha384 as u16) => Ok(Self::RsaPssRsaeSha384),
            x if x == (Self::RsaPssRsaeSha512 as u16) => Ok(Self::RsaPssRsaeSha512),
            x if x == (Self::Ed25519 as u16) => Ok(Self::Ed25519),
            x if x == (Self::Ed448 as u16) => Ok(Self::Ed448),
            x if x == (Self::RsaPssPssSha256 as u16) => Ok(Self::RsaPssPssSha256),
            x if x == (Self::RsaPssPssSha384 as u16) => Ok(Self::RsaPssPssSha384),
            x if x == (Self::RsaPssPssSha512 as u16) => Ok(Self::RsaPssPssSha512),
            x if x == (Self::RsaPkcs1Sha1 as u16) => Ok(Self::RsaPkcs1Sha1),
            x if x == (Self::EcdsaSha1 as u16) => Ok(Self::EcdsaSha1),
            x => Err(x),
        }
    }
}

/// # References
///
/// * [RFC 6066 Section 3](https://datatracker.ietf.org/doc/html/rfc6066#section-3)
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NameType {
    Hostname = 0,
}

/// # References
///
/// * [RFC 8446 Section 4.2.7](https://datatracker.ietf.org/doc/html/rfc8446#section-4.2.7)
/// * [RFC 8446 Section 9.1](https://datatracker.ietf.org/doc/html/rfc8446#section-9.1)
#[repr(u16)]
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(non_camel_case_types, dead_code)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub(crate) enum NamedGroup {
    // Elliptic Curve Groups (ECDHE)
    secp256r1 = 0x0017, // required
    secp384r1 = 0x0018,
    secp521r1 = 0x0019,
    x25519 = 0x001D,
    x448 = 0x001E,
    // Finite Field Groups (DHE)
    ffdhe2048 = 0x0100,
    ffdhe3072 = 0x0101,
    ffdhe4096 = 0x0102,
    ffdhe6144 = 0x0103,
    ffdhe8192 = 0x0104,
    // Reserved Code Points
    // ffdhe_private_use(0x01FC..0x01FF),
    // ecdhe_private_use(0xFE00..0xFEFF),
}

impl NamedGroup {
    pub const fn msb(self) -> u8 {
        ((self as u16) >> 8) as u8
    }

    pub const fn lsb(self) -> u8 {
        self as u8
    }
}

impl TryFrom<u16> for NamedGroup {
    type Error = u16;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            x if x == (Self::secp256r1 as u16) => Ok(Self::secp256r1),
            x if x == (Self::secp384r1 as u16) => Ok(Self::secp384r1),
            x if x == (Self::secp521r1 as u16) => Ok(Self::secp521r1),
            x if x == (Self::x25519 as u16) => Ok(Self::x25519),
            x if x == (Self::x448 as u16) => Ok(Self::x448),
            x if x == (Self::ffdhe2048 as u16) => Ok(Self::ffdhe2048),
            x if x == (Self::ffdhe3072 as u16) => Ok(Self::ffdhe3072),
            x if x == (Self::ffdhe4096 as u16) => Ok(Self::ffdhe4096),
            x if x == (Self::ffdhe6144 as u16) => Ok(Self::ffdhe6144),
            x if x == (Self::ffdhe8192 as u16) => Ok(Self::ffdhe8192),
            x => Err(x),
        }
    }
}

/// # References
///
/// * [RFC 8446 Section 4.2.9](https://datatracker.ietf.org/doc/html/rfc8446#section-4.2.9)
#[repr(u8)]
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub(crate) enum PskKeyExchangeMode {
    /// PSK-only key establishment.
    ///
    /// In this mode, the server MUST NOT supply a `key_share` value.
    Ke = 0,
    /// PSK with (EC)DHE key establishment.
    ///
    /// In this mode, the  client and server MUST supply `key_share` values as
    /// described in [RFC 8446 Section 4.2.8].
    ///
    /// [RFC 8446 Section 4.2.8]: https://datatracker.ietf.org/doc/html/rfc8446#section-4.2.8
    DheKe = 1,
}

/// Create a vector with up-to 2**8-1 bytes at compile time.
// N = NUM_ELEMENTS * ELEMENT_SIZE + size_of::<u8>();
const fn vector_u8<const NUM_ELEMENTS: usize, const ELEMENT_SIZE: usize, const N: usize>(
    values: [[u8; ELEMENT_SIZE]; NUM_ELEMENTS],
) -> [u8; N] {
    let mut ret: [u8; N] = [0; N];

    let length: usize = ELEMENT_SIZE * NUM_ELEMENTS;

    // fill in vector length
    ret[0] = length as u8;

    // for loops not allowed in const
    let mut value_idx: usize = 0;
    while value_idx < NUM_ELEMENTS {
        let mut value_byte_idx: usize = 0;
        while value_byte_idx < ELEMENT_SIZE {
            ret[value_idx * ELEMENT_SIZE + value_byte_idx + size_of::<u8>()] =
                values[value_idx][value_byte_idx];
            value_byte_idx += 1;
        }
        value_idx += 1;
    }

    ret
}

/// Create a vector with up-to 2**16-1 bytes at compile time.
// N = NUM_ELEMENTS * ELEMENT_SIZE + size_of::<u16>();
const fn vector_u16<const NUM_ELEMENTS: usize, const ELEMENT_SIZE: usize, const N: usize>(
    values: [[u8; ELEMENT_SIZE]; NUM_ELEMENTS],
) -> [u8; N] {
    let mut ret: [u8; N] = [0; N];

    let length: usize = ELEMENT_SIZE * NUM_ELEMENTS;
    let length: u16 = length as u16;

    let mut length_idx: usize = 0;
    while length_idx < size_of::<u16>() {
        ret[length_idx] = length.to_be_bytes()[length_idx];
        length_idx += 1;
    }

    // for loops not allowed in const
    let mut value_idx: usize = 0;
    while value_idx < NUM_ELEMENTS {
        let mut value_byte_idx: usize = 0;
        while value_byte_idx < ELEMENT_SIZE {
            ret[value_idx * ELEMENT_SIZE + value_byte_idx + size_of::<u16>()] =
                values[value_idx][value_byte_idx];
            value_byte_idx += 1;
        }
        value_idx += 1;
    }

    ret
}

/// Create a `SupportedVersions`.
///
/// # References
///
/// * [RFC 8446 Appendix B.3.1.1](https://datatracker.ietf.org/doc/html/rfc8446#appendix-B.3.1.1)
///
/// ```text
/// struct {
///     select (Handshake.msg_type) {
///         case client_hello:
///              ProtocolVersion versions<2..254>;
///
///         case server_hello: /* and HelloRetryRequest */
///              ProtocolVersion selected_version;
///     };
/// } SupportedVersions;
/// ```
// N = NUM_VERSIONS * size_of::<u16>() + size_of::<u8>();
const fn supported_versions<const NUM_VERSIONS: usize, const N: usize>(
    versions: [u16; NUM_VERSIONS],
) -> [u8; N] {
    let mut versions_bytes: [[u8; 2]; NUM_VERSIONS] = [[0; 2]; NUM_VERSIONS];
    let mut version_idx: usize = 0;
    while version_idx < NUM_VERSIONS {
        versions_bytes[version_idx] = versions[version_idx].to_be_bytes();
        version_idx += 1;
    }
    vector_u8(versions_bytes)
}

/// Create a `PskKeyExchangeModes`.
///
/// # References
///
/// * [RFC 8446 Appendix 4.2.9](https://datatracker.ietf.org/doc/html/rfc8446#section-4.2.9)
///
/// ```text
/// struct {
///     PskKeyExchangeMode ke_modes<1..255>;
/// } PskKeyExchangeModes;
/// ```
// N = NUM_MODES + size_of::<u8>();
const fn psk_key_exchange_modes<const NUM_MODES: usize, const N: usize>(
    modes: [PskKeyExchangeMode; NUM_MODES],
) -> [u8; N] {
    let mut mode_bytes: [[u8; 1]; NUM_MODES] = [[0; 1]; NUM_MODES];
    let mut mode_idx: usize = 0;
    while mode_idx < NUM_MODES {
        mode_bytes[mode_idx][0] = modes[mode_idx] as u8;
        mode_idx += 1;
    }
    vector_u8(mode_bytes)
}

/// Create a `SignatureSchemeList`.
///
/// # References
///
/// * [RFC 8446 Section 4.2.3](https://datatracker.ietf.org/doc/html/rfc8446#section-4.2.3)
///
/// ```text
/// struct {
///     SignatureScheme supported_signature_algorithms<2..2^16-2>;
/// } SignatureSchemeList;
/// ```
// N = NUM_SCHEMES * size_of::<u16>() + size_of::<u16>();
const fn signature_scheme_list<const NUM_SCHEMES: usize, const N: usize>(
    schemes: [SignatureScheme; NUM_SCHEMES],
) -> [u8; N] {
    let mut schemes_bytes: [[u8; 2]; NUM_SCHEMES] = [[0; 2]; NUM_SCHEMES];
    let mut scheme_idx: usize = 0;
    while scheme_idx < NUM_SCHEMES {
        schemes_bytes[scheme_idx] = (schemes[scheme_idx] as u16).to_be_bytes();
        scheme_idx += 1;
    }
    vector_u16(schemes_bytes)
}

/// Create a `NamedGroupList`.
///
/// # References
///
/// * [RFC 8446 Section 4.2.7](https://datatracker.ietf.org/doc/html/rfc8446#section-4.2.7)
///
/// ```text
/// struct {
///     NamedGroup named_group_list<2..2^16-1>;
/// } NamedGroupList;
/// ```
// N = N_VALUES * size_of::<u16>() + size_of::<u16>();
const fn named_group_list<const N_VALUES: usize, const N: usize>(
    values: [NamedGroup; N_VALUES],
) -> [u8; N] {
    let mut value_bytes: [[u8; 2]; N_VALUES] = [[0; 2]; N_VALUES];
    let mut value_idx: usize = 0;
    while value_idx < N_VALUES {
        value_bytes[value_idx] = (values[value_idx] as u16).to_be_bytes();
        value_idx += 1;
    }
    vector_u16(value_bytes)
}

/// Create an `Extension`.
///
/// # References
///
/// * [RFC 8446 Section 4.2](https://datatracker.ietf.org/doc/html/rfc8446#section-4.2)
///
/// ```text
/// struct {
///     ExtensionType extension_type;
///     opaque extension_data<0..2^16-1>;
/// } Extension;
/// ```
// N = DATA_LEN + size_of::<u16>() + size_of::<u16>()
const fn extension<const DATA_LEN: usize, const N: usize>(
    extension: ExtensionType,
    data: [u8; DATA_LEN],
) -> [u8; N] {
    let mut ret: [u8; N] = [0; N];
    ret[0] = (extension as u16).to_be_bytes()[0];
    ret[1] = (extension as u16).to_be_bytes()[1];
    ret[2] = (data.len() as u16).to_be_bytes()[0];
    ret[3] = (data.len() as u16).to_be_bytes()[1];

    let mut data_idx: usize = 0;
    while data_idx < DATA_LEN {
        ret[data_idx + 4] = data[data_idx];
        data_idx += 1;
    }

    ret
}

/// Create a list of cipher suites.
///
/// # References
///
/// * [RFC 8446 Section 4.1.2](https://datatracker.ietf.org/doc/html/rfc8446#section-4.1.2)
///
/// ```text
/// uint8 CipherSuite[2];    /* Cryptographic suite selector */
///
/// struct {
///     ProtocolVersion legacy_version = 0x0303;    /* TLS v1.2 */
///     Random random;
///     opaque legacy_session_id<0..32>;
///     CipherSuite cipher_suites<2..2^16-2>;
///     opaque legacy_compression_methods<1..2^8-1>;
///     Extension extensions<8..2^16-1>;
/// } ClientHello;
/// ```
// N = N_VALUES * size_of::<u16>() + size_of::<u16>();
const fn cipher_suites<const N_VALUES: usize, const N: usize>(
    values: [CipherSuite; N_VALUES],
) -> [u8; N] {
    let mut value_bytes: [[u8; 2]; N_VALUES] = [[0; 2]; N_VALUES];
    let mut value_idx: usize = 0;
    while value_idx < N_VALUES {
        value_bytes[value_idx] = values[value_idx].value();
        value_idx += 1;
    }
    vector_u16(value_bytes)
}

const CONTENT_TYPE: [u8; 1] = [ContentType::Handshake as u8];
const TLS_VERSION: [u8; 2] = (TlsVersion::V1_2 as u16).to_be_bytes();

pub const RECORD_HEADER_NO_LENGTH: [u8; CONTENT_TYPE.len() + TLS_VERSION.len()] =
    const_concat_bytes!(CONTENT_TYPE, TLS_VERSION);

const LEGACY_SESION_ID_LENGTH: [u8; 1] = [0];

const CIPHER_SUITES: [CipherSuite; 1] = [CipherSuite::TLS_AES_128_GCM_SHA256];
const CIPHER_SUITES_LIST: [u8; CIPHER_SUITES.len() * size_of::<u16>() + size_of::<u16>()] =
    cipher_suites(CIPHER_SUITES);

// length 1, value null
const LEGACY_COMPRESSION_METHODS: [u8; 2] = [1, 0];

pub const LEGACY_THINGS_AND_CIPHER_SUITES: [u8; LEGACY_SESION_ID_LENGTH.len()
    + CIPHER_SUITES_LIST.len()
    + LEGACY_COMPRESSION_METHODS.len()] = const_concat_bytes!(
    const_concat_bytes!(LEGACY_SESION_ID_LENGTH, CIPHER_SUITES_LIST),
    LEGACY_COMPRESSION_METHODS
);

const SUPPORTED_VERSIONS: [u16; 1] = [TlsVersion::V1_3 as u16];
const CLIENT_HELLO_SUPPORTED_VERSIONS: [u8; SUPPORTED_VERSIONS.len() * size_of::<u16>()
    + size_of::<u8>()] = supported_versions(SUPPORTED_VERSIONS);
const CLIENT_HELLO_SUPPORTED_VERSIONS_EXTENSION: [u8; CLIENT_HELLO_SUPPORTED_VERSIONS.len()
    + size_of::<u16>()
    + size_of::<u16>()] = extension(
    ExtensionType::SupportedVersions,
    CLIENT_HELLO_SUPPORTED_VERSIONS,
);

const SIGNATURE_SCHEMES: [SignatureScheme; 9] = [
    SignatureScheme::RsaPkcs1Sha256,
    SignatureScheme::RsaPkcs1Sha384,
    SignatureScheme::RsaPkcs1Sha512,
    SignatureScheme::EcdsaSecp256r1Sha256,
    SignatureScheme::EcdsaSecp384r1Sha384,
    SignatureScheme::RsaPssRsaeSha256,
    SignatureScheme::RsaPssRsaeSha384,
    SignatureScheme::RsaPssRsaeSha512,
    SignatureScheme::Ed25519,
];
const SIGNATURE_SCHEME_LIST: [u8; SIGNATURE_SCHEMES.len() * size_of::<u16>() + size_of::<u16>()] =
    signature_scheme_list(SIGNATURE_SCHEMES);
const SIGNATURE_ALGORITHMS_EXTENSION: [u8; SIGNATURE_SCHEME_LIST.len()
    + size_of::<u16>()
    + size_of::<u16>()] = extension(ExtensionType::SignatureAlgorithms, SIGNATURE_SCHEME_LIST);

const SUPPORTED_GROUPS: [NamedGroup; 1] = [NamedGroup::secp256r1];
const NAMED_GROUP_LIST: [u8; SUPPORTED_GROUPS.len() * size_of::<u16>() + size_of::<u16>()] =
    named_group_list(SUPPORTED_GROUPS);
const SUPPORTED_GROUPS_EXTENSION: [u8; NAMED_GROUP_LIST.len()
    + size_of::<u16>()
    + size_of::<u16>()] = extension(ExtensionType::SupportedGroups, NAMED_GROUP_LIST);

const KEY_EXCHANGE_MODES: [PskKeyExchangeMode; 1] = [PskKeyExchangeMode::DheKe];
const KEY_EXCHANGE_MODES_LIST: [u8; KEY_EXCHANGE_MODES.len() + size_of::<u8>()] =
    psk_key_exchange_modes(KEY_EXCHANGE_MODES);
const KEY_EXCHANGE_MODES_EXTENSION: [u8; KEY_EXCHANGE_MODES_LIST.len()
    + size_of::<u16>()
    + size_of::<u16>()] = extension(ExtensionType::PskKeyExchangeModes, KEY_EXCHANGE_MODES_LIST);

pub const CONST_EXTENSIONS: [u8; SUPPORTED_GROUPS_EXTENSION.len()
    + KEY_EXCHANGE_MODES_EXTENSION.len()
    + CLIENT_HELLO_SUPPORTED_VERSIONS_EXTENSION.len()
    + SIGNATURE_ALGORITHMS_EXTENSION.len()] = const_concat_bytes!(
    const_concat_bytes!(SUPPORTED_GROUPS_EXTENSION, KEY_EXCHANGE_MODES_EXTENSION),
    const_concat_bytes!(
        CLIENT_HELLO_SUPPORTED_VERSIONS_EXTENSION,
        SIGNATURE_ALGORITHMS_EXTENSION
    ),
);

struct ClientHelloWriter<'a> {
    buf: &'a mut [u8],
    len: usize,
    key_schedule: &'a mut KeySchedule,
}

impl<'a> ClientHelloWriter<'a> {
    pub fn copy_from_slice(&mut self, src: &[u8]) {
        self.copy_from_slice_no_hash(src);
        self.key_schedule.update_transcript_hash(src);
    }

    pub fn copy_from_slice_no_hash(&mut self, src: &[u8]) {
        self.buf[self.len..(self.len + src.len())].copy_from_slice(src);
        self.len += src.len();
    }

    pub fn push(&mut self, byte: u8) {
        self.buf[self.len] = byte;
        self.key_schedule.update_transcript_hash(&[byte]);
        self.len += 1;
    }

    pub fn write_binder(&mut self, psk: &[u8], truncated_transcript_hash: Sha256) {
        let binder: GenericArray<u8, U32> =
            self.key_schedule.binder(psk, truncated_transcript_hash);
        self.copy_from_slice(&binder);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn ser(
    buf: &mut [u8],
    random: &[u8; 32],
    hostname: &Hostname,
    client_public: &p256_cortex_m4::PublicKey,
    key_schedule: &mut KeySchedule,
    psk: &[u8],
    identity: &[u8],
    record_size_limit: u16,
) -> usize {
    let mut writer: ClientHelloWriter = ClientHelloWriter {
        buf,
        len: 0,
        key_schedule,
    };

    let extensions_length: u16 =
        137 + (CONST_EXTENSIONS.len() as u16) + u16::from(hostname.len()) + (identity.len() as u16);
    let handshake_length: u16 = 43 + extensions_length;
    let tls_plaintext_length: u16 = 4 + handshake_length;

    // the record header is not included in the transcript hash
    writer.copy_from_slice_no_hash(&RECORD_HEADER_NO_LENGTH);
    writer.copy_from_slice_no_hash(&tls_plaintext_length.to_be_bytes());
    let start_of_record: usize = writer.len;

    writer.push(HandshakeType::ClientHello as u8);
    writer.push(0);
    writer.copy_from_slice(&handshake_length.to_be_bytes());
    let start_of_handshake: usize = writer.len;

    writer.copy_from_slice(&u16::from(TlsVersion::V1_2).to_be_bytes());
    writer.copy_from_slice(random);
    writer.copy_from_slice(&LEGACY_THINGS_AND_CIPHER_SUITES);
    writer.copy_from_slice(&extensions_length.to_be_bytes());
    let start_of_extensions: usize = writer.len;

    writer.copy_from_slice(&CONST_EXTENSIONS);

    // server name indication
    // https://datatracker.ietf.org/doc/html/rfc6066#section-3
    {
        let hostname_len: u16 = hostname.len().into();
        let server_name_list_len: u16 = hostname_len + 3;
        let extension_len: u16 = server_name_list_len + 2;

        writer.copy_from_slice(&u16::from(ExtensionType::ServerName).to_be_bytes());
        writer.copy_from_slice(&extension_len.to_be_bytes());
        writer.copy_from_slice(&server_name_list_len.to_be_bytes());
        writer.push(NameType::Hostname as u8);
        writer.copy_from_slice(&hostname_len.to_be_bytes());
        writer.copy_from_slice(hostname.as_bytes());
    }

    // key share
    // https://datatracker.ietf.org/doc/html/rfc8446#section-4.2.8
    {
        const P256_UNCOMPRESSED_POINT_SIZE: u16 = 65;
        const CLIENT_SHARES_LEN: u16 = P256_UNCOMPRESSED_POINT_SIZE
            + (size_of::<u16>() as u16)
            + (size_of::<NamedGroup>() as u16);
        const EXTENSION_LEN: u16 = CLIENT_SHARES_LEN + (size_of::<u16>() as u16);

        const KEY_SHARE_EXTENSION_HEADER: [u8; 10] = [
            ExtensionType::KeyShare.msb(),
            ExtensionType::KeyShare.lsb(),
            (EXTENSION_LEN >> 8) as u8,
            EXTENSION_LEN as u8,
            (CLIENT_SHARES_LEN >> 8) as u8,
            CLIENT_SHARES_LEN as u8,
            NamedGroup::secp256r1.msb(),
            NamedGroup::secp256r1.lsb(),
            (P256_UNCOMPRESSED_POINT_SIZE >> 8) as u8,
            P256_UNCOMPRESSED_POINT_SIZE as u8,
        ];
        writer.copy_from_slice(&KEY_SHARE_EXTENSION_HEADER);
        writer.copy_from_slice(&client_public.to_uncompressed_sec1_bytes());
    }

    // record size limit
    // https://www.rfc-editor.org/rfc/rfc8449
    {
        writer.copy_from_slice(&u16::from(ExtensionType::RecordSizeLimit).to_be_bytes());
        writer.copy_from_slice(&2_u16.to_be_bytes());
        writer.copy_from_slice(&record_size_limit.to_be_bytes());
    }

    // pre-shared key
    // https://datatracker.ietf.org/doc/html/rfc8446#section-4.2.11
    let len: usize = {
        let identity_len: u16 = identity.len() as u16;
        let identities_len: u16 =
            (identity_len + (size_of::<u32>() as u16)) + (size_of::<u16>() as u16);
        const BINDER_LEN: u8 = 32;
        const BINDERS_LEN: u16 = (BINDER_LEN as u16) + (size_of::<u8>() as u16);
        let extension_len: u16 =
            identities_len + BINDERS_LEN + (size_of::<u16>() as u16) + (size_of::<u16>() as u16);

        // For identities established externally, an obfuscated_ticket_age of 0
        // SHOULD be used.
        const OBFUSCATED_TICKET_AGE: u32 = 0;

        writer.copy_from_slice(&u16::from(ExtensionType::PreSharedKey).to_be_bytes());
        writer.copy_from_slice(&extension_len.to_be_bytes());
        writer.copy_from_slice(&identities_len.to_be_bytes());
        writer.copy_from_slice(&identity_len.to_be_bytes());
        writer.copy_from_slice(identity);
        writer.copy_from_slice(&OBFUSCATED_TICKET_AGE.to_be_bytes());
        let truncated_transcript_hash: Sha256 = writer.key_schedule.transcript_hash();
        writer.copy_from_slice(&BINDERS_LEN.to_be_bytes());
        writer.copy_from_slice(&[BINDER_LEN]);
        writer.write_binder(psk, truncated_transcript_hash);
        writer.len
    };

    let actual_extensions_length: u16 = (len - start_of_extensions) as u16;
    assert_eq!(actual_extensions_length, extensions_length);

    let actual_handshake_length: u16 = (len - start_of_handshake) as u16;
    assert_eq!(actual_handshake_length, handshake_length);

    let actual_tls_plaintext_length: u16 = (len - start_of_record) as u16;
    assert_eq!(actual_tls_plaintext_length, tls_plaintext_length);

    len
}
