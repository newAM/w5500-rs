/// MQTT client identifier
///
/// # References
///
/// * [Client Identifier](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901059)
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "ufmt", derive(ufmt::derive::uDebug))]
pub struct ClientId<'a> {
    client_id: &'a str,
}

impl<'a> ClientId<'a> {
    /// Create a new client ID from a string.
    ///
    /// # Requirements
    ///
    /// If any requirement is not met `None` is returned.
    ///
    /// * `client_id` must not be empty
    /// * `client_id` must be 23 characters or fewer
    /// * `client_id` must only contain characters in the ranges of
    ///   `b'a'..=b'z'`, `b'A'..=b'Z'`, and `b'0'..=b'9'`
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_mqtt::ClientId;
    ///
    /// assert!(ClientId::new("valid").is_some());
    /// assert!(ClientId::new("not-valid").is_none());
    /// ```
    pub const fn new(client_id: &'a str) -> Option<Self> {
        // this is really ugly to allow `const` evaluation

        const fn char_is_valid(ch: u8) -> bool {
            (ch >= b'A' && ch <= b'Z') || (ch >= b'a' && ch <= b'z') || (ch >= b'0' && ch <= b'9')
        }

        if client_id.is_empty() || client_id.len() > 23 {
            None
        } else {
            let mut idx: usize = 0;
            while idx < client_id.len() {
                if !char_is_valid(client_id.as_bytes()[idx]) {
                    return None;
                }
                idx += 1;
            }
            Some(Self { client_id })
        }
    }

    /// Create a new client ID, panicking if the client ID is invalid.
    ///
    /// # Panics
    ///
    /// This is the same as [`new`](Self::new), but it will panic on invalid
    /// hostnames.
    ///
    /// This should only be used in `const` contexts where the evaluation will
    /// fail at compile time.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_mqtt::ClientId;
    ///
    /// const CLIENT_ID: ClientId = ClientId::new_unwrapped("valid");
    /// ```
    pub const fn new_unwrapped(client_id: &'a str) -> Self {
        match Self::new(client_id) {
            Some(client_id) => client_id,
            None => ::core::panic!("invalid client ID"),
        }
    }

    /// Length of the client ID in bytes.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_mqtt::ClientId;
    ///
    /// const CLIENT_ID: ClientId = ClientId::new_unwrapped("hello");
    ///
    /// assert_eq!(CLIENT_ID.len(), 5);
    /// ```
    #[inline]
    #[allow(clippy::len_without_is_empty)] // constructor validates client ID is not empty
    pub const fn len(&self) -> u8 {
        // truncation is safe - client ID was validated to be less than 23 chars
        self.client_id.len() as u8
    }

    /// Converts the client ID to a byte slice.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_mqtt::ClientId;
    ///
    /// const CLIENT_ID: ClientId = ClientId::new_unwrapped("hello");
    /// assert_eq!(CLIENT_ID.as_bytes(), [104, 101, 108, 108, 111]);
    /// ```
    #[inline]
    pub const fn as_bytes(&self) -> &[u8] {
        self.client_id.as_bytes()
    }
}

impl<'a> From<ClientId<'a>> for &'a str {
    #[inline]
    fn from(client_id: ClientId<'a>) -> Self {
        client_id.client_id
    }
}

#[cfg(test)]
mod test {
    use super::ClientId;

    #[test]
    fn valid_client_id() {
        ["foo", "BAR", "0", "01234567890123456789aaa"]
            .iter()
            .for_each(|client_id| {
                assert!(
                    ClientId::new(client_id).is_some(),
                    "ClientId '{client_id}' is valid"
                )
            })
    }

    #[test]
    fn invalid_client_id() {
        ["", "aaaaaaaaaaaaaaaaaaaaaaaa", "foo-bar", "🙃"]
            .iter()
            .for_each(|client_id| {
                assert!(
                    ClientId::new(client_id).is_none(),
                    "ClientId '{client_id}' is invalid"
                )
            })
    }
}
