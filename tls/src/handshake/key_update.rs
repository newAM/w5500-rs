/// Key Update Request
///
/// # References
///
/// * [RFC 8446 Section 4.6.3](https://datatracker.ietf.org/doc/html/rfc8446#section-4.6.3)
///
/// ```text
/// enum {
///     update_not_requested(0), update_requested(1), (255)
/// } KeyUpdateRequest;
/// ```
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "ufmt", derive(ufmt::derive::uDebug))]
pub enum KeyUpdateRequest {
    UpdateNotRequested = 0,
    UpdateRequested = 1,
}

impl From<KeyUpdateRequest> for u8 {
    #[inline]
    fn from(kur: KeyUpdateRequest) -> Self {
        kur as u8
    }
}

impl TryFrom<u8> for KeyUpdateRequest {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            x if x == (Self::UpdateNotRequested as u8) => Ok(Self::UpdateNotRequested),
            x if x == (Self::UpdateRequested as u8) => Ok(Self::UpdateRequested),
            x => Err(x),
        }
    }
}
