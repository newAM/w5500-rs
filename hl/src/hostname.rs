/// A validated hostname.
///
/// This is not used within this crate, it is provided here for crates
/// implementing protocols such as DNS and DHCP to use.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Hostname<'a> {
    hostname: &'a str,
}

#[allow(clippy::len_without_is_empty)] // empty is not allowed by `new`
impl<'a> Hostname<'a> {
    /// Create a new hostname.
    ///
    /// This validates the hostname for [RFC-1035] compliance:
    ///
    /// A hostname is valid if the following condition are true:
    ///
    /// - It does not start or end with `-` or `.`.
    /// - It does not contain any characters outside of the alphanumeric range, except for `-` and `.`.
    /// - It is not empty.
    /// - It is 253 or fewer characters.
    /// - Its labels (characters separated by `.`) are not empty.
    /// - Its labels are 63 or fewer characters.
    /// - Its lables do not start or end with '-' or '.'.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_hl::Hostname;
    ///
    /// assert!(Hostname::new("is-valid-example").is_some());
    /// assert!(Hostname::new("this-is-not-?-valid").is_none());
    /// ```
    ///
    /// [RFC-1035]: https://www.rfc-editor.org/rfc/rfc1035
    pub fn new(hostname: &'a str) -> Option<Self> {
        // Adapted from hostname-validator: https://github.com/pop-os/hostname-validator
        // see: https://github.com/pop-os/hostname-validator/issues/2
        fn is_valid_char(byte: u8) -> bool {
            (b'a'..=b'z').contains(&byte)
                || (b'A'..=b'Z').contains(&byte)
                || (b'0'..=b'9').contains(&byte)
                || byte == b'-'
                || byte == b'.'
        }

        if hostname.is_empty()
            || hostname.len() > 253
            || hostname.bytes().any(|byte| !is_valid_char(byte))
            || hostname.split('.').any(|label| {
                label.is_empty()
                    || label.len() > 63
                    || label.ends_with('-')
                    || label.starts_with('-')
            })
        {
            None
        } else {
            Some(Self { hostname })
        }
    }

    /// Returns an iterator over the labels of the hostname.
    ///
    /// # Example
    ///
    /// ```
    /// use core::str::Split;
    /// use w5500_hl::Hostname;
    ///
    /// let docs_rs: Hostname = Hostname::new("docs.rs").unwrap();
    /// let mut lables: Split<char> = docs_rs.labels();
    ///
    /// assert_eq!(lables.next(), Some("docs"));
    /// assert_eq!(lables.next(), Some("rs"));
    /// assert_eq!(lables.next(), None);
    /// ```
    #[inline]
    pub fn labels(&self) -> core::str::Split<'a, char> {
        self.hostname.split('.')
    }

    /// Length of the hostname in bytes.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_hl::Hostname;
    ///
    /// let docs_rs: Hostname = Hostname::new("docs.rs").unwrap();
    ///
    /// assert_eq!(docs_rs.len(), 7);
    /// ```
    #[inline]
    pub fn len(&self) -> u8 {
        // truncation is OK, hostname is validated to be 255 bytes or fewer
        self.hostname.len() as u8
    }

    /// Create a new hostname without checking for validity.
    ///
    /// # Safety
    ///
    /// The `hostname` argument must meet all the conditions for validity
    /// described in [`new`](Self::new).
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_hl::Hostname;
    ///
    /// // safety: doc.rs is a valid hostname
    /// const DOCS_RS: Hostname = unsafe { Hostname::new_unchecked("docs.rs") };
    /// ```
    #[allow(unsafe_code)]
    #[inline]
    pub const unsafe fn new_unchecked(hostname: &'a str) -> Self {
        Self { hostname }
    }

    /// Converts the hostname to a byte slice.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_hl::Hostname;
    ///
    /// let docs_rs: Hostname = Hostname::new("docs.rs").unwrap();
    /// assert_eq!(docs_rs.as_bytes(), [100, 111, 99, 115, 46, 114, 115]);
    /// ```
    #[inline]
    pub const fn as_bytes(&self) -> &[u8] {
        self.hostname.as_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::Hostname;

    #[test]
    fn valid_hostnames() {
        for hostname in &[
            "VaLiD-HoStNaMe",
            "50-name",
            "235235",
            "example.com",
            "VaLid.HoStNaMe",
            "123.456",
        ] {
            assert!(Hostname::new(hostname).is_some(), "{hostname} is not valid");
        }
    }

    #[test]
    fn invalid_hostnames() {
        for hostname in &[
            "-invalid-name",
            "also-invalid-",
            "asdf@fasd",
            "@asdfl",
            "asd f@",
            ".invalid",
            "invalid.name.",
            "invalid.-starting.char",
            "invalid.ending-.char",
            "empty..label",
            "label-is-way-to-longgggggggggggggggggggggggggggggggggggggggggggg.com",
        ] {
            assert!(
                Hostname::new(hostname).is_none(),
                "{hostname} should not be valid"
            );
        }
    }
}
