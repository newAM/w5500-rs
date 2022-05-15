/// SNTP timestamp format.
///
/// # References
///
/// * [RFC 4330 Section 3](https://datatracker.ietf.org/doc/html/rfc4330#section-3)
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Timestamp {
    pub(crate) bits: u64,
}

impl Timestamp {
    #[cfg_attr(not(any(feature = "chrono", feature = "time")), allow(dead_code))]
    fn new(secs: u64, nanos: u64) -> Self {
        let upper: u32 = secs
            .try_into()
            .unwrap_or_else(|_| secs.saturating_sub(1 << 32).try_into().unwrap_or(u32::MAX));
        let lower: u32 = ((nanos * u64::from(u32::MAX)) / 1_000_000_000) as u32;
        Self {
            bits: ((upper as u64) << 32) | (lower as u64),
        }
    }

    #[cfg_attr(not(any(feature = "chrono", feature = "time")), allow(dead_code))]
    #[must_use]
    fn secs(&self) -> i64 {
        let seconds_bits: u32 = (self.bits >> 32) as u32;
        // If bit 0 is set, the UTC time is in the range 1968-2036
        if seconds_bits & 0x8000_0000 != 0 {
            i64::from(seconds_bits)
        } else {
            // If bit 0 is not set, the time is in the range 2036-2104 and
            // UTC time is reckoned from 6h 28m 16s UTC on 7 February 2036.
            i64::from(seconds_bits) + i64::from(u32::MAX) + 1
        }
    }

    #[cfg_attr(not(any(feature = "chrono", feature = "time")), allow(dead_code))]
    #[must_use]
    fn nanos(&self) -> u32 {
        // safe to truncate, number is always less than u32::MAX
        ((self.bits & 0xFFFF_FFFF) * 1_000_000_000 / u64::from(u32::MAX)) as u32
    }

    /// Raw bits of the timestamp value.
    #[must_use]
    pub const fn to_bits(self) -> u64 {
        self.bits
    }

    /// Returns `true` if the timestamp is zero.
    #[must_use]
    pub const fn is_zero(&self) -> bool {
        self.bits == 0
    }
}

#[cfg(feature = "chrono")]
fn origin_chrono() -> chrono::NaiveDateTime {
    chrono::NaiveDate::from_ymd(1900, 1, 1).and_hms(0, 0, 0)
}

/// Returned upon a failed conversion to or from [`Timestamp`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct TimestampError(pub(crate) ());

#[cfg(feature = "chrono")]
impl TryFrom<chrono::naive::NaiveDateTime> for Timestamp {
    type Error = TimestampError;

    fn try_from(ndt: chrono::naive::NaiveDateTime) -> Result<Self, Self::Error> {
        let elapsed: chrono::Duration = ndt.signed_duration_since(origin_chrono());
        let secs: i64 = elapsed.num_seconds();
        let nanos: u64 = elapsed
            .checked_sub(&chrono::Duration::seconds(secs))
            .unwrap_or_else(|| chrono::Duration::seconds(0))
            .num_nanoseconds()
            .unwrap_or(0)
            .try_into()
            .map_err(|_| TimestampError(()))?;
        let secs: u64 = secs.try_into().map_err(|_| TimestampError(()))?;

        Ok(Self::new(secs, nanos))
    }
}

#[cfg(feature = "chrono")]
impl TryFrom<Timestamp> for chrono::naive::NaiveDateTime {
    type Error = TimestampError;

    fn try_from(timestamp: Timestamp) -> Result<Self, Self::Error> {
        origin_chrono()
            .checked_add_signed(chrono::Duration::seconds(timestamp.secs()))
            .ok_or(TimestampError(()))?
            .checked_add_signed(chrono::Duration::nanoseconds(timestamp.nanos().into()))
            .ok_or(TimestampError(()))
    }
}

#[cfg(feature = "time")]
impl TryFrom<Timestamp> for time::PrimitiveDateTime {
    type Error = TimestampError;

    fn try_from(timestamp: Timestamp) -> Result<Self, Self::Error> {
        const ORIGIN: time::PrimitiveDateTime = {
            const DATE: time::Date =
                match time::Date::from_calendar_date(1900, time::Month::January, 1) {
                    Ok(date) => date,
                    Err(_) => ::core::panic!("invalid date"),
                };
            const TIME: time::Time = match time::Time::from_hms(0, 0, 0) {
                Ok(time) => time,
                Err(_) => ::core::panic!("invalid time"),
            };
            time::PrimitiveDateTime::new(DATE, TIME)
        };

        ORIGIN
            .checked_add(time::Duration::seconds(timestamp.secs()))
            .ok_or(TimestampError(()))?
            .checked_add(time::Duration::nanoseconds(timestamp.nanos().into()))
            .ok_or(TimestampError(()))
    }
}

#[cfg(test)]
mod tests {
    use super::Timestamp;
    use chrono::naive::{NaiveDate, NaiveDateTime, NaiveTime};
    use time::PrimitiveDateTime;

    #[test]
    fn chrono() {
        let timestamp: Timestamp = Timestamp {
            bits: 0xe5_fd_82_24_23_ec_4b_12,
        };

        let ndt: NaiveDateTime = timestamp.try_into().unwrap();

        let expected_date: NaiveDate = NaiveDate::from_ymd(2022, 04, 10);
        let expected_time: NaiveTime = NaiveTime::from_hms_nano(16, 19, 48, 140324298);
        let expected_datetime: NaiveDateTime = NaiveDateTime::new(expected_date, expected_time);

        core::assert_eq!(ndt, expected_datetime);

        let timestamp_converted: Timestamp = ndt.try_into().unwrap();

        core::assert_eq!(timestamp.secs(), timestamp_converted.secs());
        let nanos_diff: u32 = timestamp.nanos().abs_diff(timestamp_converted.nanos());
        core::assert!(nanos_diff <= 1, "nanos_diff={nanos_diff}");
    }

    #[test]
    fn time() {
        let timestamp: Timestamp = Timestamp {
            bits: 0xe5_fd_82_24_23_ec_4b_12,
        };

        let pdt: PrimitiveDateTime = timestamp.try_into().unwrap();

        core::assert_eq!(pdt.year(), 2022);
        core::assert_eq!(pdt.month(), time::Month::April);
        core::assert_eq!(pdt.day(), 10);
        core::assert_eq!(pdt.hour(), 16);
        core::assert_eq!(pdt.minute(), 19);
        core::assert_eq!(pdt.second(), 48);
    }

    #[test]
    fn chrono_zero() {
        let timestamp: Timestamp = Timestamp { bits: 0 };

        let ndt: NaiveDateTime = timestamp.try_into().unwrap();
        let expected: NaiveDateTime = NaiveDate::from_ymd(2036, 2, 7).and_hms(6, 28, 16);

        core::assert_eq!(ndt, expected);
    }

    #[test]
    fn time_zero() {
        let timestamp: Timestamp = Timestamp { bits: 0 };

        let date: time::Date =
            time::Date::from_calendar_date(2036, time::Month::February, 7).unwrap();
        let time: time::Time = time::Time::from_hms(6, 28, 16).unwrap();
        let expected: PrimitiveDateTime = PrimitiveDateTime::new(date, time);

        let pdt: PrimitiveDateTime = timestamp.try_into().unwrap();

        core::assert_eq!(pdt, expected);
    }
}
