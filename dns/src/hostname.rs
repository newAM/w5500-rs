// Adapted from hostname-validator: https://github.com/pop-os/hostname-validator
// with additional length checks for DNS and DHCP requirements.

#[cfg(feature = "defmt")]
use dfmt as defmt;

/// A validated hostname.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Hostname<'a> {
    hostname: &'a str,
}

impl<'a> Hostname<'a> {
    /// Create a new hostname.
    ///
    /// This validates the hostname for [RFC-1123] compliance:
    ///
    /// A hostname is valid if the following condition are true:
    ///
    /// - It does not start or end with `-` or `.`.
    /// - It does not contain any characters outside of the alphanumeric range, except for `-` and `.`.
    /// - It is less than or equal to 255 bytes.
    /// - It is greater than 0 byte.
    /// - All labels (characters separated by `.`) are 63 or fewer bytes.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_dns::Hostname;
    ///
    /// assert!(Hostname::new("is-valid-example").is_some());
    /// assert!(Hostname::new("this-is-not-?-valid").is_none());
    /// ```
    ///
    /// [RFC-1123]: https://datatracker.ietf.org/doc/html/rfc1123
    pub fn new(hostname: &'a str) -> Option<Self> {
        fn is_valid_char(byte: u8) -> bool {
            (byte >= b'a' && byte <= b'z')
                || (byte >= b'A' && byte <= b'Z')
                || (byte >= b'0' && byte <= b'9')
                || byte == b'-'
                || byte == b'.'
        }

        if hostname.ends_with('-')
            || hostname.starts_with('-')
            || hostname.ends_with('.')
            || hostname.starts_with('.')
            || hostname.is_empty()
            || hostname.len() > 255
            || hostname.bytes().any(|byte| !is_valid_char(byte))
            || hostname.split('.').any(|label| label.len() > 63)
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
    /// use w5500_dns::Hostname;
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
    /// use w5500_dns::Hostname;
    ///
    /// let docs_rs: Hostname = Hostname::new("docs.rs").unwrap();
    ///
    /// assert_eq!(docs_rs.len(), 7);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.hostname.len()
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
    /// use w5500_dns::Hostname;
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
    /// use w5500_dns::Hostname;
    ///
    /// let docs_rs: Hostname = Hostname::new("docs.rs").unwrap();
    /// assert_eq!(docs_rs.as_bytes(), [100, 111, 99, 115, 46, 114, 115]);
    /// ```
    #[inline]
    pub const fn as_bytes(&self) -> &[u8] {
        self.hostname.as_bytes()
    }
}

impl<'a> From<Hostname<'a>> for &'a str {
    #[inline]
    fn from(hn: Hostname<'a>) -> Self {
        hn.hostname
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
            "label-is-way-to-longgggggggggggggggggggggggggggggggggggggggggggg.com",
        ] {
            assert!(
                Hostname::new(hostname).is_none(),
                "{hostname} should not be valid"
            );
        }
    }

    #[test]
    fn invalid_hostname_really_long() {
        const EXPECTED_LEN: usize = 256;
        let mut long_hosname: String = String::with_capacity(EXPECTED_LEN);

        for i in 0..16 {
            for _ in 0..15 {
                long_hosname.push('a');
            }
            if i != 15 {
                long_hosname.push('.');
            }
        }
        long_hosname.push('a');

        assert_eq!(long_hosname.len(), EXPECTED_LEN);
        assert!(Hostname::new(&long_hosname).is_none());

        long_hosname.pop();
        assert!(Hostname::new(&long_hosname).is_some());
    }
}
