use crate::{
    write_variable_byte_integer, CtrlPkt, FILTER_LEN_LEN, PACKET_ID_LEN, PROPERTY_LEN_LEN,
};
use w5500_hl::{io::Write, Error as HlError};

/// Subscribe Acknowledgment Codes
///
/// # References
///
/// * [SUBACK Payload](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901178)
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum SubAckReasonCode {
    /// Granted QoS 0
    ///
    /// The subscription is accepted and the maximum QoS sent will be QoS 0. This might be a lower QoS than was requested.
    QoS0 = 0x00,
    /// Granted QoS 1
    ///
    /// The subscription is accepted and the maximum QoS sent will be QoS 1. This might be a lower QoS than was requested.
    QoS1 = 0x01,
    /// Granted QoS 2
    ///
    /// The subscription is accepted and any received QoS will be sent to this subscription.
    QoS2 = 0x02,
    /// Unspecified error
    ///
    /// The subscription is not accepted and the Server either does not wish to reveal the reason or none of the other Reason Codes apply.
    UnspecifiedError = 0x80,
    /// Implementation specific error
    ///
    /// The SUBSCRIBE is valid but the Server does not accept it.
    ImplementationSpecificError = 0x83,
    /// Not authorized
    ///
    /// The Client is not authorized to make this subscription.
    NotAuthorized = 0x87,
    /// Topic Filter invalid
    ///
    /// The Topic Filter is correctly formed but is not allowed for this Client.
    TopicFilterInvalid = 0x8F,
    /// Packet Identifier in use
    ///
    /// The specified Packet Identifier is already in use.
    PacketIdentifierInUse = 0x91,
    /// Quota exceeded
    ///
    /// An implementation or administrative imposed limit has been exceeded.
    QuotaExceeded = 0x97,
    /// Shared Subscriptions not supported
    ///
    /// The Server does not support Shared Subscriptions for this Client.
    SharedSubscriptionsNotSupported = 0x9E,
    /// Subscription Identifiers not supported
    ///
    /// The Server does not support Subscription Identifiers; the subscription is not accepted.
    SubscriptionIdentifiersNotSupported = 0xA1,
    /// Wildcard Subscriptions not supported
    ///
    /// The Server does not support Wildcard Subscriptions; the subscription is not accepted.
    WildcardSubscriptionsNotSupported = 0xA2,
}

impl TryFrom<u8> for SubAckReasonCode {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            x if (x == Self::QoS0 as u8) => Ok(Self::QoS0),
            x if (x == Self::QoS1 as u8) => Ok(Self::QoS1),
            x if (x == Self::QoS2 as u8) => Ok(Self::QoS2),
            x if (x == Self::UnspecifiedError as u8) => Ok(Self::UnspecifiedError),
            x if (x == Self::ImplementationSpecificError as u8) => {
                Ok(Self::ImplementationSpecificError)
            }
            x if (x == Self::NotAuthorized as u8) => Ok(Self::NotAuthorized),
            x if (x == Self::TopicFilterInvalid as u8) => Ok(Self::TopicFilterInvalid),
            x if (x == Self::PacketIdentifierInUse as u8) => Ok(Self::PacketIdentifierInUse),
            x if (x == Self::QuotaExceeded as u8) => Ok(Self::QuotaExceeded),
            x if (x == Self::SharedSubscriptionsNotSupported as u8) => {
                Ok(Self::SharedSubscriptionsNotSupported)
            }
            x if (x == Self::SubscriptionIdentifiersNotSupported as u8) => {
                Ok(Self::SubscriptionIdentifiersNotSupported)
            }
            x if (x == Self::WildcardSubscriptionsNotSupported as u8) => {
                Ok(Self::WildcardSubscriptionsNotSupported)
            }
            x => Err(x),
        }
    }
}

/// Unsubscribe Acknowledgment Codes
///
/// # References
///
/// * [UNSUBACK Payload](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901194)
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum UnSubAckReasonCode {
    /// The subscription is deleted.
    Success = 0x00,
    /// No matching Topic Filter is being used by the Client.
    NoSubscriptionExisted = 0x11,
    /// The unsubscribe could not be completed and the Server either does not
    /// wish to reveal the reason or none of the other Reason Codes apply.
    UnspecifiedError = 0x80,
    /// The UNSUBSCRIBE is valid but the Server does not accept it.
    ImplementationSpecificError = 0x83,
    /// The Client is not authorized to unsubscribe.
    NotAuthorized = 0x87,
    /// The Topic Filter is correctly formed but is not allowed for this Client.
    TopicFilterInvalid = 0x8F,
    /// The specified Packet Identifier is already in use.
    PacketIdentifierInUse = 0x91,
}

impl TryFrom<u8> for UnSubAckReasonCode {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            x if (x == Self::Success as u8) => Ok(Self::Success),
            x if (x == Self::NoSubscriptionExisted as u8) => Ok(Self::NoSubscriptionExisted),
            x if (x == Self::UnspecifiedError as u8) => Ok(Self::UnspecifiedError),
            x if (x == Self::ImplementationSpecificError as u8) => {
                Ok(Self::ImplementationSpecificError)
            }
            x if (x == Self::NotAuthorized as u8) => Ok(Self::NotAuthorized),
            x if (x == Self::TopicFilterInvalid as u8) => Ok(Self::TopicFilterInvalid),
            x if (x == Self::PacketIdentifierInUse as u8) => Ok(Self::PacketIdentifierInUse),
            x => Err(x),
        }
    }
}

/// `SUBSCRIBE` acknowledgment
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SubAck {
    /// Packet Identifier.
    ///
    /// This can be compared with the return value of [`Client::subscribe`] to
    /// determine which subscribe is being acknowledged.
    ///
    /// [`Client::subscribe`]: crate::Client::subscribe
    pub pkt_id: u16,
    /// SUBACK reason code.
    ///
    /// This should be checked to ensure the SUBSCRIBE was successful.
    pub code: SubAckReasonCode,
}

/// `UNSUBSCRIBE` acknowledgment
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct UnSubAck {
    /// Packet Identifier.
    ///
    /// This can be compared with the return value of [`Client::unsubscribe`]
    /// to determine which unsubscribe is being acknowledged.
    ///
    /// [`Client::unsubscribe`]: crate::Client::unsubscribe
    pub pkt_id: u16,
    /// UNSUBACK reason code.
    ///
    /// This should be checked to ensure the UNSUBSCRIBE was successful.
    pub code: UnSubAckReasonCode,
}

pub fn send_subscribe<E, Writer: Write<E>>(
    mut writer: Writer,
    filter: &str,
    pkt_id: u16,
) -> Result<u16, HlError<E>> {
    if filter.is_empty() {
        Ok(0)
    } else {
        const SUBSCRIPTION_OPTIONS_LEN: u16 = 1;

        let filter_len: u16 = (filter.len() as u16) + FILTER_LEN_LEN + SUBSCRIPTION_OPTIONS_LEN;

        let remaining_len: u32 =
            PACKET_ID_LEN + u32::from(PROPERTY_LEN_LEN) + u32::from(filter_len);

        writer.write_all(&[(CtrlPkt::SUBSCRIBE as u8) << 4 | 0b0010])?;
        write_variable_byte_integer(&mut writer, remaining_len)?;
        writer.write_all(&[
            // packet identifier
            (pkt_id >> 8) as u8,
            pkt_id as u8,
            // property length
            0,
        ])?;

        writer.write_all(
            u16::try_from(filter.len())
                .unwrap_or(u16::MAX)
                .to_be_bytes()
                .as_ref(),
        )?;
        writer.write_all(filter.as_bytes())?;
        // subscription options flags
        // 00 => reserved
        // 10 => retain handling: do not set messages at subscribtion time
        // 0 => retain as published: all messages have the retain flag cleared
        // 1 => no local option: do not send messages published by this client
        // 00 => QoS 0: at most once delivery
        writer.write_all(&[0b00100100])?;

        writer.send()?;

        Ok(pkt_id)
    }
}

pub fn send_unsubscribe<E, Writer: Write<E>>(
    mut writer: Writer,
    filter: &str,
    pkt_id: u16,
) -> Result<u16, HlError<E>> {
    if filter.is_empty() {
        Ok(0)
    } else {
        let filter_len: u16 = (filter.len() as u16) + FILTER_LEN_LEN;

        let remaining_len: u32 =
            PACKET_ID_LEN + u32::from(PROPERTY_LEN_LEN) + u32::from(filter_len);

        writer.write_all(&[(CtrlPkt::UNSUBSCRIBE as u8) << 4 | 0b0010])?;
        write_variable_byte_integer(&mut writer, remaining_len)?;
        writer.write_all(&[
            // packet identifier
            (pkt_id >> 8) as u8,
            pkt_id as u8,
            // property length
            0,
        ])?;

        writer.write_all(
            u16::try_from(filter.len())
                .unwrap_or(u16::MAX)
                .to_be_bytes()
                .as_ref(),
        )?;
        writer.write_all(filter.as_bytes())?;

        writer.send()?;

        Ok(pkt_id)
    }
}
