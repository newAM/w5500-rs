pub mod client_hello;
mod encrypted_extensions;
mod finished;
mod key_update;
mod server_hello;

pub(crate) use encrypted_extensions::recv_encrypted_extensions;
pub use finished::client_finished;
pub use key_update::KeyUpdateRequest;
pub(crate) use server_hello::recv_server_hello;

/// Handshake Type.
///
/// # References
///
/// * [RFC 8446 Section 4](https://datatracker.ietf.org/doc/html/rfc8446#section-4)
///
/// ```text
/// enum {
///     client_hello(1),
///     server_hello(2),
///     new_session_ticket(4),
///     end_of_early_data(5),
///     encrypted_extensions(8),
///     certificate(11),
///     certificate_request(13),
///     certificate_verify(15),
///     finished(20),
///     key_update(24),
///     message_hash(254),
///     (255)
/// } HandshakeType;
/// ```
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "ufmt", derive(ufmt::derive::uDebug))]
pub enum HandshakeType {
    ClientHello = 1,
    ServerHello = 2,
    NewSessionTicket = 4,
    EndOfEarlyData = 5,
    EncryptedExtensions = 8,
    Certificate = 11,
    CertificateRequest = 13,
    CertificateVerify = 15,
    Finished = 20,
    KeyUpdate = 24,
    // this is a synthetic handshake message and will not be TX'd or RX'd
    // https://datatracker.ietf.org/doc/html/rfc8446#section-4.4.1
    // MessageHash = 254,
}

impl From<HandshakeType> for u8 {
    #[inline]
    fn from(handshake_type: HandshakeType) -> Self {
        handshake_type as u8
    }
}

impl TryFrom<u8> for HandshakeType {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            x if x == (Self::ClientHello as u8) => Ok(Self::ClientHello),
            x if x == (Self::ServerHello as u8) => Ok(Self::ServerHello),
            x if x == (Self::NewSessionTicket as u8) => Ok(Self::NewSessionTicket),
            x if x == (Self::EndOfEarlyData as u8) => Ok(Self::EndOfEarlyData),
            x if x == (Self::EncryptedExtensions as u8) => Ok(Self::EncryptedExtensions),
            x if x == (Self::Certificate as u8) => Ok(Self::Certificate),
            x if x == (Self::CertificateRequest as u8) => Ok(Self::CertificateRequest),
            x if x == (Self::CertificateVerify as u8) => Ok(Self::CertificateVerify),
            x if x == (Self::Finished as u8) => Ok(Self::Finished),
            x if x == (Self::KeyUpdate as u8) => Ok(Self::KeyUpdate),
            // x if x == (Self::MessageHash as u8) => Ok(Self::MessageHash),
            _ => Err(value),
        }
    }
}

/// # References
///
/// * [RFC 8446 Section 4](https://datatracker.ietf.org/doc/html/rfc8446#section-4)
///
/// ```text
/// struct {
///     HandshakeType msg_type;    /* handshake type */
///     uint24 length;             /* bytes in message */
///     select (Handshake.msg_type) {
///         case client_hello:          ClientHello;
///         case server_hello:          ServerHello;
///         case end_of_early_data:     EndOfEarlyData;
///         case encrypted_extensions:  EncryptedExtensions;
///         case certificate_request:   CertificateRequest;
///         case certificate:           Certificate;
///         case certificate_verify:    CertificateVerify;
///         case finished:              Finished;
///         case new_session_ticket:    NewSessionTicket;
///         case key_update:            KeyUpdate;
///     };
/// } Handshake;
/// ```
pub struct HandshakeHeader {
    buf: [u8; Self::LEN],
}

impl From<[u8; Self::LEN]> for HandshakeHeader {
    fn from(buf: [u8; Self::LEN]) -> Self {
        Self { buf }
    }
}

impl HandshakeHeader {
    pub const LEN: usize = 4;

    pub fn msg_type(&self) -> Result<HandshakeType, u8> {
        self.buf[0].try_into()
    }

    pub fn length(&self) -> u32 {
        u32::from_be_bytes(self.buf) & 0x00FF_FFFF
    }

    pub fn length_with_header(&self) -> u32 {
        self.length() + (Self::LEN as u32)
    }

    pub fn as_bytes(&self) -> &[u8; Self::LEN] {
        &self.buf
    }
}
