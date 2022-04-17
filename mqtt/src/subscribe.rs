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
    ImplementationError = 0x83,
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
    PackedIdentifierInUse = 0x91,
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
            x if (x == Self::ImplementationError as u8) => Ok(Self::ImplementationError),
            x if (x == Self::NotAuthorized as u8) => Ok(Self::NotAuthorized),
            x if (x == Self::TopicFilterInvalid as u8) => Ok(Self::TopicFilterInvalid),
            x if (x == Self::PackedIdentifierInUse as u8) => Ok(Self::PackedIdentifierInUse),
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
