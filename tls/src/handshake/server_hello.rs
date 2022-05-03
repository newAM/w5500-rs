use crate::{
    cipher_suites::CipherSuite, io::CircleReader, AlertDescription, ExtensionType, NamedGroup,
    TlsVersion,
};
const P256_KEY_LEN: usize = 65;

/// Server Hello key exchange message.
///
/// # References
///
/// * [RFC 8446 Appendix B.3.1](https://datatracker.ietf.org/doc/html/rfc8446#appendix-B.3.1)
///
/// ```text
/// struct {
///     ProtocolVersion legacy_version = 0x0303;    /* TLS v1.2 */
///     Random random;
///     opaque legacy_session_id_echo<0..32>;
///     CipherSuite cipher_suite;
///     uint8 legacy_compression_method = 0;
///     Extension extensions<6..2^16-1>;
/// } ServerHello;
/// ```
pub(crate) fn recv_server_hello(
    reader: &mut CircleReader,
) -> Result<p256::PublicKey, AlertDescription> {
    let legacy_version: u16 = reader.next_u16()?;
    const EXPECTED_LEGACY_VERSION: u16 = TlsVersion::V1_2 as u16;
    if legacy_version != EXPECTED_LEGACY_VERSION {
        error!(
            "expected legacy_version {:04X} got {:04X}",
            EXPECTED_LEGACY_VERSION, legacy_version
        );
        return Err(AlertDescription::ProtocolVersion);
    }

    // random value
    reader.skip_n(32)?;

    let session_id_len: u8 = reader.next_u8()?;
    if session_id_len != 0 {
        error!("session ID length is not 0: {}", session_id_len);
        return Err(AlertDescription::IllegalParameter);
    }

    let cipher_suite: [u8; 2] = reader.next_n()?;
    let cipher_suite: CipherSuite = cipher_suite
        .try_into()
        .map_err(|_| AlertDescription::IllegalParameter)?;
    if cipher_suite != CipherSuite::TLS_AES_128_GCM_SHA256 {
        error!("unsupported cipher suite: {:?}", cipher_suite);
        return Err(AlertDescription::HandshakeFailure);
    }

    let compression_method: u8 = reader.next_u8()?;
    if compression_method != 0 {
        error!("compression method is not 0: {}", compression_method);
        return Err(AlertDescription::IllegalParameter);
    }

    let extensions_len: u16 = reader.next_u16()?;
    let extensions_end: u16 = match reader.stream_position().checked_add(extensions_len) {
        Some(end) => end,
        None => {
            error!("ServerHello extentions len exceeds record len");
            return Err(AlertDescription::DecodeError);
        }
    };

    // required extension checklist
    let mut done_supported_versions: bool = false;
    let mut done_key_share: bool = false;
    let mut done_pre_shared_key: bool = false;

    let mut key_buf: [u8; 65] = [0; 65];

    while extensions_end > reader.stream_position() {
        let extension_type: ExtensionType = match ExtensionType::try_from(reader.next_u16()?) {
            Ok(extension_type) => extension_type,
            Err(x) => {
                error!("illegal extension type: {:#04X}", x);
                return Err(AlertDescription::IllegalParameter);
            }
        };

        let extension_len: u16 = reader.next_u16()?;

        debug!("ServerHello {:?} length {}", extension_type, extension_len);

        let extension_start: u16 = reader.stream_position();

        match extension_type {
            ExtensionType::KeyShare => {
                if done_key_share {
                    error!("KeyShare appeared twice");
                    return Err(AlertDescription::IllegalParameter);
                }

                let group: Result<NamedGroup, u16> = reader.next_u16()?.try_into();
                if group != Ok(NamedGroup::secp256r1) {
                    // should never occur because we inform the server
                    // that we only support for secp256r1
                    error!("unsupported KeyShareEntry.group={:?}", group);
                    return Err(AlertDescription::IllegalParameter);
                }

                let key_exchange_len: u16 = reader.next_u16()?;

                if usize::from(key_exchange_len) != P256_KEY_LEN {
                    error!(
                        "expected P256 key length {} got {}",
                        P256_KEY_LEN, key_exchange_len
                    );
                    return Err(AlertDescription::DecodeError);
                }

                reader.read_exact(&mut key_buf)?;

                done_key_share = true;
            }
            ExtensionType::SupportedVersions => {
                if done_supported_versions {
                    error!("SupportedVersions appeared twice");
                    return Err(AlertDescription::IllegalParameter);
                }

                let selected_version: u16 = reader.next_u16()?;
                const EXPECTED_VERSION: u16 = TlsVersion::V1_3 as u16;

                if selected_version != EXPECTED_VERSION {
                    error!("Unsupported TLS version: {:?}", selected_version);
                    // https://datatracker.ietf.org/doc/html/rfc8446#section-4.2.1
                    // If the "supported_versions" extension in the
                    // ServerHello contains a version not offered by the
                    // client or contains a version prior to TLS 1.3, the
                    // client MUST abort the handshake with an
                    // "illegal_parameter" alert.
                    return Err(AlertDescription::IllegalParameter);
                }
                done_supported_versions = true;
            }
            ExtensionType::PreSharedKey => {
                const EXPECTED_LEN: u16 = 2;
                if extension_len != EXPECTED_LEN {
                    error!(
                        "expected PreSharedKey length {} got {}",
                        EXPECTED_LEN, extension_len
                    );
                    return Err(AlertDescription::DecodeError);
                }

                // at the moment we can only send one identity so the server
                // can only select identity 0
                let selected_identity: u16 = reader.next_u16()?;
                if selected_identity != 0 {
                    error!("expected selected_identity 0 got {}", selected_identity);
                    return Err(AlertDescription::DecodeError);
                }
                done_pre_shared_key = true;
            }
            // https://datatracker.ietf.org/doc/html/rfc8446#section-4.2
            // All others are not allowed for server hello
            x => {
                error!("illegal or unknown extension for ServerHello: {:?}", x);
                return Err(AlertDescription::UnsupportedExtension);
            }
        }

        let n_read: u16 = reader.stream_position() - extension_start;

        if extension_len != n_read {
            error!(
                "{:?} extension length {} != n_read {}",
                extension_type, extension_len, n_read
            );
            return Err(AlertDescription::DecodeError);
        }
    }

    if !done_key_share {
        error!("missing key share extension");
        return Err(AlertDescription::MissingExtension);
    }

    if !done_supported_versions {
        error!("missing supported versions extension");
        return Err(AlertDescription::MissingExtension);
    }

    if !done_pre_shared_key {
        error!("missing pre-shared key extension");
        return Err(AlertDescription::MissingExtension);
    }

    match p256::PublicKey::from_sec1_bytes(&key_buf) {
        Ok(public_key) => Ok(public_key),
        Err(_e) => {
            #[cfg(feature = "log")]
            log::error!("P256 public key decode {:?}", _e);
            #[cfg(feature = "defmt")]
            defmt::error!("P256 public key decode {}", defmt::Debug2Format(&_e));
            Err(AlertDescription::DecodeError)
        }
    }
}
