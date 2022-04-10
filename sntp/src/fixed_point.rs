/// A fixed point value.
///
/// NTP uses 32-bit numbers with a decimal between bits 15 and 16 for some
/// values.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FixedPoint {
    pub(crate) bits: u32,
}

impl FixedPoint {
    /// Raw bits of the fixed point value.
    #[must_use]
    pub const fn to_bits(self) -> u32 {
        self.bits
    }

    const fn raw_whole(&self) -> u32 {
        self.bits >> 16
    }

    const fn raw_fractional(&self) -> u32 {
        self.bits & 0xFFFF
    }
}

#[cfg(feature = "num-rational")]
impl From<FixedPoint> for num_rational::Ratio<u32> {
    fn from(fp: FixedPoint) -> Self {
        Self::new_raw(
            fp.raw_whole() * (u16::MAX as u32) + fp.raw_fractional(),
            u16::MAX as u32,
        )
    }
}

impl From<FixedPoint> for f32 {
    fn from(fp: FixedPoint) -> Self {
        (fp.raw_whole() as f32) + ((fp.raw_fractional() as f32) / (u16::MAX as f32))
    }
}

impl From<FixedPoint> for f64 {
    fn from(fp: FixedPoint) -> Self {
        (fp.raw_whole() as f64) + ((fp.raw_fractional() as f64) / (u16::MAX as f64))
    }
}

#[cfg(test)]
mod tests {
    use super::FixedPoint;
    use num_rational::Ratio;

    #[test]
    fn ratio() {
        let ratio: Ratio<u32> = FixedPoint { bits: 0x0000_0009 }.into();
        const EXPECTED: Ratio<u32> = Ratio::new_raw(0x09, u16::MAX as u32);
        core::assert_eq!(ratio, EXPECTED);
    }

    #[test]
    fn f32() {
        let val: f32 = FixedPoint { bits: 0x0000_0009 }.into();
        core::assert!(
            val > 0.000_136 && val < 0.000_138,
            "ratio should be 0.000_137 seconds, not {val}"
        );
    }

    #[test]
    fn f64() {
        let val: f64 = FixedPoint { bits: 0x0000_0009 }.into();
        core::assert!(
            val > 0.000_136 && val < 0.000_138,
            "ratio should be 0.000_137 seconds, not {val}"
        );
    }
}
