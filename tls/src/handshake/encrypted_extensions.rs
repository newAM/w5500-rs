use crate::{AlertDescription, ExtensionType, io::CircleReader};
use core::cmp::min;

/// Encrypted extensions message.
///
/// # References
///
/// * [RFC 8446 Section 4.3.1](https://datatracker.ietf.org/doc/html/rfc8446#section-4.3.1)
///
/// ```text
/// struct {
///     Extension extensions<0..2^16-1>;
/// } EncryptedExtensions;
/// ```
pub(crate) fn recv_encrypted_extensions(reader: &mut CircleReader) -> Result<(), AlertDescription> {
    let extensions_len: u16 = reader.next_u16()?;
    let extensions_end: u16 = match reader.stream_position().checked_add(extensions_len) {
        Some(end) => end,
        None => {
            error!("EncryptedExtensions extentions len exceeds record len");
            return Err(AlertDescription::DecodeError);
        }
    };

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

        // 9 possible extensions
        match extension_type {
            ExtensionType::ServerName => {
                // 253 is maximum length for a valid DNS name
                let mut buf: [u8; 253] = [0; 253];
                let read_len: u16 = min(buf.len() as u16, extension_len);
                reader.read_exact(&mut buf[..read_len.into()])?;

                warn!(
                    "server_name is unused: {:?}",
                    core::str::from_utf8(&buf[..read_len.into()])
                );

                // RFCs are weird and there are valid hostnames longer than DNS
                // names
                let remain: u16 = extension_len - read_len;
                if remain > 0 {
                    reader.skip_n(remain)?;
                }
            }
            ExtensionType::MaxFragmentLength => {
                // server should not send this since we do not include
                // it in the ClientHello
                error!("Unexpected MaxFragmentLength");
                return Err(AlertDescription::UnsupportedExtension);
            }
            ExtensionType::SupportedGroups => {
                // Clients MUST NOT act upon any information
                // found in "supported_groups" prior to successful completion of the
                // handshake but MAY use the information learned from a successfully
                // completed handshake to change what groups they use in their
                // "key_share" extension in subsequent connections.
                debug!("ignoring SupportedGroups");
                reader.skip_n(extension_len)?;
            }
            ExtensionType::UseSrtp => {
                // only used for DTLS
                error!("Unexpected use_strp extension");
                return Err(AlertDescription::UnsupportedExtension);
            }
            ExtensionType::Heartbeat => {
                // only used for DTLS
                error!("Unexpected heartbeat extension");
                return Err(AlertDescription::UnsupportedExtension);
            }
            ExtensionType::ApplicationLayerProtocolNegotiation => {
                warn!("application_layer_protocol_negotiation is unused");
                reader.skip_n(extension_len)?;
            }
            ExtensionType::ClientCertificateType => {
                // only used for DTLS
                error!("Unexpected client_certificate_type extension");
                return Err(AlertDescription::UnsupportedExtension);
            }
            ExtensionType::ServerCertificateType => {
                // only used for DTLS
                error!("Unexpected server_certificate_type extension");
                return Err(AlertDescription::UnsupportedExtension);
            }
            ExtensionType::EarlyData => {
                // server will only send this if we send early data, which
                // is not yet supported
                error!("Unexpected early_data extension");
                return Err(AlertDescription::UnsupportedExtension);
            }
            x => {
                error!("Extension invalid for EncryptedExtensions: {:?}", x);
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

    Ok(())
}
