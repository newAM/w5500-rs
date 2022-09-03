/// [Properties](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901027)
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "ufmt", derive(ufmt::derive::uDebug))]
#[repr(u8)]
#[non_exhaustive]
#[allow(dead_code)]
pub enum Properties {
    /// Payload format indicator
    PayloadFormatIndicator = 0x01,
    /// Maximum packet size
    MaxPktSize = 0x27,
}
