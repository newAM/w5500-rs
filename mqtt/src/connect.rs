use w5500_hl::{io::Write, Error as HlError};

use crate::{properties::Properties, ClientId, CtrlPkt};

/// [Connect Reason Codes](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901079)
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum ConnectReasonCode {
    /// The Server does not wish to reveal the reason for the failure, or none of the other Reason Codes apply.
    UnspecifiedError = 0x80,
    /// Data within the CONNECT packet could not be correctly parsed.
    MalformedPacket = 0x81,
    /// Data in the CONNECT packet does not conform to this specification.
    ProtocolError = 0x82,
    /// The CONNECT is valid but is not accepted by this Server.
    ImplementationSpecificError = 0x83,
    /// The Server does not support the version of the MQTT protocol requested by the Client.
    UnsupportedProtocolVersion = 0x84,
    /// The Client Identifier is a valid string but is not allowed by the Server.
    ClientIdentifierNotValid = 0x85,
    /// The Server does not accept the User Name or Password specified by the Client
    BadUserNameOrPassword = 0x86,
    /// The Client is not authorized to connect.
    NotAuthorized = 0x87,
    /// The MQTT Server is not available.
    ServerUnavailable = 0x88,
    /// The Server is busy. Try again later.
    ServerBusy = 0x89,
    /// This Client has been banned by administrative action. Contact the server administrator.
    Banned = 0x8A,
    /// The authentication method is not supported or does not match the authentication method currently in use.
    BadAuthenticationMethod = 0x8C,
    /// The Will Topic Name is not malformed, but is not accepted by this Server.
    TopicNameInvalid = 0x90,
    /// The CONNECT packet exceeded the maximum permissible size.
    PacketTooLarge = 0x95,
    /// An implementation or administrative imposed limit has been exceeded.
    QuotaExceeded = 0x97,
    /// The Will Payload does not match the specified Payload Format Indicator.
    PayloadFormatInvalid = 0x99,
    /// The Server does not support retained messages, and Will Retain was set to 1.
    RetainNotSupported = 0x9A,
    /// The Server does not support the QoS set in Will QoS.
    QoSNotSupported = 0x9B,
    /// The Client should temporarily use another server.
    UseAnotherServer = 0x9C,
    /// The Client should permanently use another server.
    ServerMoved = 0x9D,
    /// The connection rate limit has been exceeded
    ConnectionRateExceeded = 0x9F,
}

impl TryFrom<u8> for ConnectReasonCode {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            x if x == (Self::UnspecifiedError as u8) => Ok(Self::UnspecifiedError),
            x if x == (Self::MalformedPacket as u8) => Ok(Self::MalformedPacket),
            x if x == (Self::ProtocolError as u8) => Ok(Self::ProtocolError),
            x if x == (Self::ImplementationSpecificError as u8) => {
                Ok(Self::ImplementationSpecificError)
            }
            x if x == (Self::UnsupportedProtocolVersion as u8) => {
                Ok(Self::UnsupportedProtocolVersion)
            }
            x if x == (Self::ClientIdentifierNotValid as u8) => Ok(Self::ClientIdentifierNotValid),
            x if x == (Self::BadUserNameOrPassword as u8) => Ok(Self::BadUserNameOrPassword),
            x if x == (Self::NotAuthorized as u8) => Ok(Self::NotAuthorized),
            x if x == (Self::ServerUnavailable as u8) => Ok(Self::ServerUnavailable),
            x if x == (Self::ServerBusy as u8) => Ok(Self::ServerBusy),
            x if x == (Self::Banned as u8) => Ok(Self::Banned),
            x if x == (Self::BadAuthenticationMethod as u8) => Ok(Self::BadAuthenticationMethod),
            x if x == (Self::TopicNameInvalid as u8) => Ok(Self::TopicNameInvalid),
            x if x == (Self::PacketTooLarge as u8) => Ok(Self::PacketTooLarge),
            x if x == (Self::QuotaExceeded as u8) => Ok(Self::QuotaExceeded),
            x if x == (Self::PayloadFormatInvalid as u8) => Ok(Self::PayloadFormatInvalid),
            x if x == (Self::RetainNotSupported as u8) => Ok(Self::RetainNotSupported),
            x if x == (Self::QoSNotSupported as u8) => Ok(Self::QoSNotSupported),
            x if x == (Self::UseAnotherServer as u8) => Ok(Self::UseAnotherServer),
            x if x == (Self::ServerMoved as u8) => Ok(Self::ServerMoved),
            x if x == (Self::ConnectionRateExceeded as u8) => Ok(Self::ConnectionRateExceeded),
            x => Err(x),
        }
    }
}

#[repr(u8)]
enum ConnectionFlags {
    CleanStart = 0x02,
    Password = 0x40,
    Username = 0x80,
}

#[derive(Debug)]
pub struct LoginCredentials<'a> {
    username: &'a str,
    password: &'a str,
}

impl<'a> LoginCredentials<'a> {
    pub fn new(username: &'a str, password: &'a str) -> Self {
        Self { username, password }
    }
}

pub fn send_connect<E, Writer: Write<E>>(
    mut writer: Writer,
    client_id: &Option<ClientId>,
    login_credentials: &Option<LoginCredentials>,
    rx_max: u16,
) -> Result<(), HlError<E>> {
    const KEEP_ALIVE: u16 = 15 * 60;

    let mut flags: u8 = ConnectionFlags::CleanStart as u8;

    let client_id = client_id.as_ref();
    let client_id_len = client_id.map_or(0, ClientId::len);

    let mut suffix_len: u8 = client_id_len;

    if let Some(login_credentials) = login_credentials {
        flags += ConnectionFlags::Username as u8;
        flags += ConnectionFlags::Password as u8;

        suffix_len += payload_len(login_credentials.username);
        suffix_len += payload_len(login_credentials.password);
    }

    #[rustfmt::skip]
    writer.write_all(&[
        // control packet type
        (CtrlPkt::CONNECT as u8) << 4,
        // remaining length
        18 + suffix_len,
        // protocol name length
        0, 4,
        // protocol name
        b'M', b'Q', b'T', b'T',
        // protocol version
        5,
        // flags, clean start is set
        flags,
        // keepalive
        (KEEP_ALIVE >> 8) as u8, KEEP_ALIVE as u8,
        // properties length
        5,
        // receive maximum property
        (Properties::MaxPktSize as u8), 0, 0, (rx_max >> 8) as u8, rx_max as u8,
        // client ID length
        0, client_id_len,
    ])?;
    if let Some(client_id) = client_id {
        writer.write_all(client_id.as_bytes())?;
    }

    if let Some(login_credentials) = login_credentials {
        let LoginCredentials { username, password } = login_credentials;
        writer.write_all(str_len_msb_lsb(username).as_slice())?;
        writer.write_all(username.as_bytes())?;
        writer.write_all(str_len_msb_lsb(password).as_slice())?;
        writer.write_all(password.as_bytes())?;
    }

    writer.send()?;
    Ok(())
}

fn payload_len(s: &str) -> u8 {
    // str len + 2 bytes for str len prefix
    (s.len() + 2) as u8
}

fn str_len_msb_lsb(s: &str) -> [u8; 2] {
    let len: u16 = s.len() as u16;
    let msb: u8 = (len >> 8) as u8;
    let lsb: u8 = len as u8;

    [msb, lsb]
}
