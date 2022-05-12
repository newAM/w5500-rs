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
    /// - It does not start or end with `'-'` or `'.'`.
    /// - It does not contain any characters outside of the alphanumeric range, except for `'-'` and `'.'`.
    /// - It is not empty.
    /// - It is 253 or fewer characters.
    /// - Its labels (characters separated by `.`) are not empty.
    /// - Its labels are 63 or fewer characters.
    /// - Its labels do not start or end with `'-'` or `'.'`.
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
    pub const fn new(hostname: &'a str) -> Option<Self> {
        // This function is very ugly because of const limitations on stable
        // for the refined non-const version see TryFrom<&str> below.

        const fn is_valid_char(byte: u8) -> bool {
            (byte >= b'a' && byte <= b'z')
                || (byte >= b'A' && byte <= b'Z')
                || (byte >= b'0' && byte <= b'9')
                || byte == b'-'
                || byte == b'.'
        }

        if hostname.is_empty() || hostname.len() > 253 {
            return None;
        }

        const fn is_valid_segment(hostname: &str, start: usize, end: usize) -> bool {
            let segment_length: usize = end - start;
            if segment_length == 0 || segment_length > 63 {
                return false;
            }

            let first_byte_label: u8 = hostname.as_bytes()[start];
            if first_byte_label == b'-' {
                return false;
            }

            let last_byte_label: u8 = hostname.as_bytes()[end - 1];
            if last_byte_label == b'-' {
                return false;
            }

            true
        }

        let mut idx: usize = 0;
        let mut segment_start: usize = 0;
        while idx < hostname.len() {
            let byte: u8 = hostname.as_bytes()[idx];
            if !is_valid_char(byte) {
                return None;
            }
            if byte == b'.' {
                if !is_valid_segment(hostname, segment_start, idx) {
                    return None;
                }

                segment_start = idx + 1;
            }
            idx += 1;
        }

        if !is_valid_segment(hostname, segment_start, idx) {
            return None;
        }

        Some(Self { hostname })
    }

    /// Create a new hostname, panicking if the hostname is invalid.
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
    /// use w5500_hl::Hostname;
    ///
    /// const MY_HOSTNAME: Hostname = Hostname::new_unwrapped("valid.hostname");
    /// ```
    pub const fn new_unwrapped(hostname: &'a str) -> Self {
        match Self::new(hostname) {
            Some(hostname) => hostname,
            None => ::core::panic!("invalid hostname"),
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
    /// const DOCS_RS: Hostname = Hostname::new_unwrapped("docs.rs");
    /// let mut lables: Split<char> = DOCS_RS.labels();
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
    /// const DOCS_RS: Hostname = Hostname::new_unwrapped("docs.rs");
    ///
    /// assert_eq!(DOCS_RS.len(), 7);
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
    /// const DOCS_RS: Hostname = Hostname::new_unwrapped("docs.rs");
    /// assert_eq!(DOCS_RS.as_bytes(), [100, 111, 99, 115, 46, 114, 115]);
    /// ```
    #[inline]
    pub const fn as_bytes(&self) -> &[u8] {
        self.hostname.as_bytes()
    }
}

/// The error type returned when a str to [`Hostname`] conversion fails.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct TryFromStrError(pub(crate) ());

impl<'a> TryFrom<&'a str> for Hostname<'a> {
    type Error = TryFromStrError;

    fn try_from(hostname: &'a str) -> Result<Self, Self::Error> {
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
            Err(TryFromStrError(()))
        } else {
            Ok(Self { hostname })
        }
    }
}

impl<'a> From<Hostname<'a>> for &'a str {
    #[inline]
    fn from(hostname: Hostname<'a>) -> Self {
        hostname.hostname
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
            "one-byte.a.label",
        ] {
            assert!(Hostname::new(hostname).is_some(), "{hostname} is not valid");
            assert!(
                Hostname::try_from(*hostname).is_ok(),
                "{hostname} is not valid"
            );
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
            "..empty-starting-label",
            "empty-ending-label..",
            "label-is-way-to-longgggggggggggggggggggggggggggggggggggggggggggg.com",
        ] {
            assert!(
                Hostname::new(hostname).is_none(),
                "{hostname} should not be valid"
            );
            assert!(
                Hostname::try_from(*hostname).is_err(),
                "{hostname} should not be valid"
            );
        }
    }
}
