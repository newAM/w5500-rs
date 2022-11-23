//! Variable data length implementation of the [`Registers`] trait using
//! an infallible SPI bus, and an infallible GPIO pin.
//!
//! This uses the W5500 variable data length mode (VDM).
//! In VDM mode the SPI frame data length is determined by the chip select pin.
//! This is the preferred blocking implementation if your W5500 has an
//! infallible chip select pin.
//!
//! [`Registers`]: crate::Registers

use crate::spi::{vdm_header, AccessMode};
use core::convert::Infallible;
use eh0::digital::v2::OutputPin;

/// W5500 blocking variable data length implementation.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct W5500<SPI, CS> {
    /// SPI bus.
    spi: SPI,
    /// GPIO for chip select.
    cs: CS,
}

impl<SPI, CS> W5500<SPI, CS>
where
    SPI: eh0::blocking::spi::Transfer<u8, Error = Infallible>
        + eh0::blocking::spi::Write<u8, Error = Infallible>,
    CS: OutputPin<Error = Infallible>,
{
    /// Creates a new `W5500` driver from a SPI peripheral and a chip select
    /// digital I/O pin.
    ///
    /// # Safety
    ///
    /// The chip select pin must be high before being passed to this function.
    ///
    /// # Example
    ///
    /// ```
    /// # struct Moo {};
    /// # impl eh0::digital::v2::OutputPin for Moo {
    /// #     type Error = core::convert::Infallible;
    /// #     fn set_low(&mut self) -> Result<(), Self::Error> { Ok(()) }
    /// #     fn set_high(&mut self) -> Result<(), Self::Error> { Ok(()) }
    /// # }
    /// # impl eh0::blocking::spi::Transfer<u8> for Moo {
    /// #     type Error = core::convert::Infallible;
    /// #     fn transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Self::Error> { Ok(words) }
    /// # }
    /// # impl eh0::blocking::spi::Write<u8> for Moo {
    /// #     type Error = core::convert::Infallible;
    /// #     fn write<'w>(&mut self, words: &[u8]) -> Result<(), Self::Error> { Ok(()) }
    /// # }
    /// # let mut pin = Moo {};
    /// # let mut spi = Moo {};
    /// use eh0::digital::v2::OutputPin;
    /// use w5500_ll::eh0::vdm_infallible::W5500;
    ///
    /// pin.set_high().unwrap();
    /// let mut w5500: W5500<_, _> = W5500::new(spi, pin);
    /// ```
    #[inline]
    #[allow(clippy::unnecessary_safety_doc)]
    pub fn new(spi: SPI, cs: CS) -> Self {
        W5500 { spi, cs }
    }

    /// Free the SPI bus and CS pin from the W5500.
    ///
    /// # Example
    ///
    /// ```
    /// # struct Moo {};
    /// # impl eh0::digital::v2::OutputPin for Moo {
    /// #     type Error = core::convert::Infallible;
    /// #     fn set_low(&mut self) -> Result<(), Self::Error> { Ok(()) }
    /// #     fn set_high(&mut self) -> Result<(), Self::Error> { Ok(()) }
    /// # }
    /// # impl eh0::blocking::spi::Transfer<u8> for Moo {
    /// #     type Error = core::convert::Infallible;
    /// #     fn transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Self::Error> { Ok(words) }
    /// # }
    /// # impl eh0::blocking::spi::Write<u8> for Moo {
    /// #     type Error = core::convert::Infallible;
    /// #     fn write<'w>(&mut self, words: &[u8]) -> Result<(), Self::Error> { Ok(()) }
    /// # }
    /// # let mut pin = Moo {};
    /// # let mut spi = Moo {};
    /// use eh0::digital::v2::OutputPin;
    /// use w5500_ll::eh0::vdm_infallible::W5500;
    ///
    /// pin.set_high().unwrap();
    /// let mut w5500: W5500<_, _> = W5500::new(spi, pin);
    /// let (spi, pin) = w5500.free();
    /// ```
    #[inline]
    pub fn free(self) -> (SPI, CS) {
        (self.spi, self.cs)
    }
}

impl<SPI, CS> crate::Registers for W5500<SPI, CS>
where
    SPI: eh0::blocking::spi::Transfer<u8, Error = Infallible>
        + eh0::blocking::spi::Write<u8, Error = Infallible>,
    CS: OutputPin<Error = Infallible>,
{
    /// SPI IO error type.
    type Error = Infallible;

    /// Read from the W5500.
    #[inline]
    fn read(&mut self, address: u16, block: u8, data: &mut [u8]) -> Result<(), Self::Error> {
        let header = vdm_header(address, block, AccessMode::Read);
        self.cs.set_low().unwrap();
        self.spi.write(&header).unwrap();
        self.spi.transfer(data).unwrap();
        self.cs.set_high()
    }

    /// Write to the W5500.
    #[inline]
    fn write(&mut self, address: u16, block: u8, data: &[u8]) -> Result<(), Self::Error> {
        let header = vdm_header(address, block, AccessMode::Write);
        self.cs.set_low().unwrap();
        self.spi.write(&header).unwrap();
        self.spi.write(data).unwrap();
        self.cs.set_high()
    }
}
