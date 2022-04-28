/// Alert level.
///
/// # References
///
/// * [RFC 8446 Section 6](https://datatracker.ietf.org/doc/html/rfc8446#section-6)
///
/// ```text
/// enum { warning(1), fatal(2), (255) } AlertLevel;
/// ```
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AlertLevel {
    /// Warning.
    Warning = 1,
    /// Fatal.
    ///
    /// Also used for unknown [`AlertLevel`] values.
    Fatal = 2,
}

impl From<AlertLevel> for u8 {
    #[inline]
    fn from(alert_level: AlertLevel) -> Self {
        alert_level as u8
    }
}

impl TryFrom<u8> for AlertLevel {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            x if x == (Self::Warning as u8) => Ok(Self::Warning),
            x if x == (Self::Fatal as u8) => Ok(Self::Fatal),
            _ => Err(value),
        }
    }
}

/// Alert description.
///
/// # References
///
/// * [RFC 8446 Section 6](https://datatracker.ietf.org/doc/html/rfc8446#section-6)
/// * [RFC 8446 Section 6.1](https://datatracker.ietf.org/doc/html/rfc8446#section-6.1)
/// * [RFC 8446 Section 6.2](https://datatracker.ietf.org/doc/html/rfc8446#section-6.2)
///
/// ```text
/// enum {
///     close_notify(0),
///     unexpected_message(10),
///     bad_record_mac(20),
///     record_overflow(22),
///     handshake_failure(40),
///     bad_certificate(42),
///     unsupported_certificate(43),
///     certificate_revoked(44),
///     certificate_expired(45),
///     certificate_unknown(46),
///     illegal_parameter(47),
///     unknown_ca(48),
///     access_denied(49),
///     decode_error(50),
///     decrypt_error(51),
///     protocol_version(70),
///     insufficient_security(71),
///     internal_error(80),
///     inappropriate_fallback(86),
///     user_canceled(90),
///     missing_extension(109),
///     unsupported_extension(110),
///     unrecognized_name(112),
///     bad_certificate_status_response(113),
///     unknown_psk_identity(115),
///     certificate_required(116),
///     no_application_protocol(120),
///     (255)
/// } AlertDescription;
/// ```
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AlertDescription {
    /// This alert notifies the recipient that the sender will
    /// not send any more messages on this connection.  Any data received
    /// after a closure alert has been received MUST be ignored.
    CloseNotify = 0,
    /// An inappropriate message (e.g., the wrong
    /// handshake message, premature Application Data, etc.) was received.
    /// This alert should never be observed in communication between
    /// proper implementations.
    UnexpectedMessage = 10,
    /// This alert is returned if a record is received which
    /// cannot be deprotected.  Because AEAD algorithms combine decryption
    /// and verification, and also to avoid side-channel attacks, this
    /// alert is used for all deprotection failures.  This alert should
    /// never be observed in communication between proper implementations,
    /// except when messages were corrupted in the network.
    BadRecordMac = 20,
    /// A TLSCiphertext record was received that had a
    /// length more than `2^14 + 256` bytes, or a record decrypted to a
    /// TLSPlaintext record with more than `2^14` bytes (or some other
    /// negotiated limit).  This alert should never be observed in
    /// communication between proper implementations, except when messages
    /// were corrupted in the network.
    RecordOverflow = 22,
    /// Receipt of a `handshake_failure` alert message
    /// indicates that the sender was unable to negotiate an acceptable
    /// set of security parameters given the options available.
    HandshakeFailure = 40,
    /// A certificate was corrupt, contained signatures
    /// that did not verify correctly, etc.
    BadCertificate = 42,
    /// A certificate was of an unsupported type.
    UnsupportedCertificate = 43,
    /// A certificate was revoked by its signer.
    CertificateRevoked = 44,
    /// A certificate has expired or is not currently valid.
    CertificateExpired = 45,
    /// Some other (unspecified) issue arose in
    /// processing the certificate, rendering it unacceptable.
    CertificateUnknown = 46,
    /// A field in the handshake was incorrect or
    /// inconsistent with other fields.  This alert is used for errors
    /// which conform to the formal protocol syntax but are otherwise
    /// incorrect.
    IllegalParameter = 47,
    /// A valid certificate chain or partial chain was received,
    /// but the certificate was not accepted because the CA certificate
    /// could not be located or could not be matched with a known trust
    /// anchor.
    UnknownCa = 48,
    /// A valid certificate or PSK was received, but when
    /// access control was applied, the sender decided not to proceed with
    /// negotiation.
    AccessDenied = 49,
    /// A message could not be decoded because some field was
    /// out of the specified range or the length of the message was
    /// incorrect.  This alert is used for errors where the message does
    /// not conform to the formal protocol syntax.  This alert should
    /// never be observed in communication between proper implementations,
    /// except when messages were corrupted in the network.
    DecodeError = 50,
    /// A handshake (not record layer) cryptographic
    /// operation failed, including being unable to correctly verify a
    /// signature or validate a Finished message or a PSK binder.
    DecryptError = 51,
    /// The protocol version the peer has attempted to
    /// negotiate is recognized but not supported.
    ProtocolVersion = 70,
    /// Returned instead of `handshake_failure` when
    /// a negotiation has failed specifically because the server requires
    /// parameters more secure than those supported by the client.
    InsufficientSecurity = 71,
    /// An internal error unrelated to the peer or the
    /// correctness of the protocol (such as a memory allocation failure)
    /// makes it impossible to continue.
    InternalError = 80,
    /// Sent by a server in response to an invalid
    /// connection retry attempt from a client (see [RFC 7507]).
    ///
    /// [RFC 7507]: https://datatracker.ietf.org/doc/html/rfc7507
    InappropriateFallback = 86,
    /// This alert notifies the recipient that the sender is
    /// canceling the handshake for some reason unrelated to a protocol
    /// failure.  If a user cancels an operation after the handshake is
    /// complete, just closing the connection by sending a `close_notify`
    /// is more appropriate.  This alert SHOULD be followed by a
    /// `close_notify`.  This alert generally has [`AlertLevel::Warning`].
    UserCanceled = 90,
    /// Sent by endpoints that receive a handshake
    /// message not containing an extension that is mandatory to send for
    /// the offered TLS version or other negotiated parameters.
    MissingExtension = 109,
    /// Sent by endpoints receiving any handshake
    /// message containing an extension known to be prohibited for
    /// inclusion in the given handshake message, or including any
    /// extensions in a `ServerHello` or `Certificate` not first offered in
    /// the corresponding `ClientHello` or `CertificateRequest`.
    UnsupportedExtension = 110,
    /// Sent by servers when no server exists identified
    /// by the name provided by the client via the `server_name` extension
    /// (see [RFC 6066]).
    ///
    /// [RFC 6066]: https://datatracker.ietf.org/doc/html/rfc6066
    UnrecognizedName = 112,
    /// Sent by clients when an invalid or
    /// unacceptable OCSP response is provided by the server via the
    /// `status_request` extension (see [RFC 6066]).
    ///
    /// [RFC 6066]: https://datatracker.ietf.org/doc/html/rfc6066
    BadCertificateStatusResponse = 113,
    /// Sent by servers when PSK key establishment is
    /// desired but no acceptable PSK identity is provided by the client.
    /// Sending this alert is OPTIONAL; servers MAY instead choose to send
    /// a `decrypt_error` alert to merely indicate an invalid PSK
    /// identity.
    UnknownPskIdentity = 115,
    /// Sent by servers when a client certificate is
    /// desired but none was provided by the client.
    CertificateRequired = 116,
    /// Sent by servers when a client
    /// `application_layer_protocol_negotiation` extension advertises only
    /// protocols that the server does not support (see [RFC 7301]).
    ///
    /// [RFC 7301]: https://datatracker.ietf.org/doc/html/rfc7301
    NoApplicationProtocol = 120,
}

impl From<AlertDescription> for u8 {
    #[inline]
    fn from(alert_description: AlertDescription) -> Self {
        alert_description as u8
    }
}

impl TryFrom<u8> for AlertDescription {
    type Error = u8;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            x if x == (Self::CloseNotify as u8) => Ok(Self::CloseNotify),
            x if x == (Self::UnexpectedMessage as u8) => Ok(Self::UnexpectedMessage),
            x if x == (Self::BadRecordMac as u8) => Ok(Self::BadRecordMac),
            x if x == (Self::RecordOverflow as u8) => Ok(Self::RecordOverflow),
            x if x == (Self::HandshakeFailure as u8) => Ok(Self::HandshakeFailure),
            x if x == (Self::BadCertificate as u8) => Ok(Self::BadCertificate),
            x if x == (Self::UnsupportedCertificate as u8) => Ok(Self::UnsupportedCertificate),
            x if x == (Self::CertificateRevoked as u8) => Ok(Self::CertificateRevoked),
            x if x == (Self::CertificateExpired as u8) => Ok(Self::CertificateExpired),
            x if x == (Self::CertificateUnknown as u8) => Ok(Self::CertificateUnknown),
            x if x == (Self::IllegalParameter as u8) => Ok(Self::IllegalParameter),
            x if x == (Self::UnknownCa as u8) => Ok(Self::UnknownCa),
            x if x == (Self::AccessDenied as u8) => Ok(Self::AccessDenied),
            x if x == (Self::DecodeError as u8) => Ok(Self::DecodeError),
            x if x == (Self::DecryptError as u8) => Ok(Self::DecryptError),
            x if x == (Self::ProtocolVersion as u8) => Ok(Self::ProtocolVersion),
            x if x == (Self::InsufficientSecurity as u8) => Ok(Self::InsufficientSecurity),
            x if x == (Self::InternalError as u8) => Ok(Self::InternalError),
            x if x == (Self::InappropriateFallback as u8) => Ok(Self::InappropriateFallback),
            x if x == (Self::UserCanceled as u8) => Ok(Self::UserCanceled),
            x if x == (Self::MissingExtension as u8) => Ok(Self::MissingExtension),
            x if x == (Self::UnsupportedExtension as u8) => Ok(Self::UnsupportedExtension),
            x if x == (Self::UnrecognizedName as u8) => Ok(Self::UnrecognizedName),
            x if x == (Self::BadCertificateStatusResponse as u8) => {
                Ok(Self::BadCertificateStatusResponse)
            }
            x if x == (Self::UnknownPskIdentity as u8) => Ok(Self::UnknownPskIdentity),
            x if x == (Self::CertificateRequired as u8) => Ok(Self::CertificateRequired),
            x if x == (Self::NoApplicationProtocol as u8) => Ok(Self::NoApplicationProtocol),
            _ => Err(value),
        }
    }
}

impl AlertDescription {
    pub(crate) fn map_w5500<E>(e: w5500_hl::Error<E>) -> Self {
        match e {
            w5500_hl::Error::UnexpectedEof => AlertDescription::DecodeError,
            w5500_hl::Error::OutOfMemory => AlertDescription::InternalError,
            w5500_hl::Error::Other(_) => AlertDescription::InternalError,
            // technically unreachable, but this can occur if there is
            // a bit flip on the SPI bus
            w5500_hl::Error::WouldBlock => {
                error!("W5500 unexpectedly blocked");
                AlertDescription::InternalError
            }
        }
    }
}

/// TLS Alert.
///
/// See [`AlertLevel`] and [`AlertDescription`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Alert {
    /// Alert level.
    pub level: AlertLevel,
    /// Alert description.
    pub description: AlertDescription,
}

impl Alert {
    pub(crate) fn new_fatal(description: AlertDescription) -> Self {
        Self {
            level: AlertLevel::Warning,
            description,
        }
    }

    pub(crate) fn new_warning(description: AlertDescription) -> Self {
        Self {
            level: AlertLevel::Warning,
            description,
        }
    }
}
