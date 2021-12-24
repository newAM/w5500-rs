//! Variable data length implementation of the [`Registers`] trait using the
//! [`embedded-hal`] blocking SPI trait, and an infallible GPIO pin.
//!
//! This uses the W5500 variable data length mode (VDM).
//! In VDM mode the SPI frame data length is determined by the chip select pin.
//! This is the preferred blocking implementation if your W5500 has an
//! infallible chip select pin.
//!
//! [`embedded-hal`]: https://github.com/rust-embedded/embedded-hal
//! [`Registers`]: crate::Registers

use crate::spi::{vdm_header, AccessMode};
use embedded_hal::digital::v2::OutputPin;

/// W5500 blocking variable data length implementation.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct W5500<SPI, CS> {
    /// SPI bus.
    spi: SPI,
    /// GPIO for chip select.
    cs: CS,
}

impl<SPI, CS, SpiError> W5500<SPI, CS>
where
    SPI: embedded_hal::blocking::spi::Transfer<u8, Error = SpiError>
        + embedded_hal::blocking::spi::Write<u8, Error = SpiError>,
    CS: OutputPin<Error = core::convert::Infallible>,
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
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[]);
    /// # struct Pin {};
    /// # impl embedded_hal::digital::v2::OutputPin for Pin {
    /// #     type Error = core::convert::Infallible;
    /// #     fn set_low(&mut self) -> Result<(), Self::Error> { Ok(()) }
    /// #     fn set_high(&mut self) -> Result<(), Self::Error> { Ok(()) }
    /// # }
    /// # let mut pin = Pin {};
    /// use embedded_hal::digital::v2::OutputPin;
    /// use w5500_ll::blocking::vdm_infallible_gpio::W5500;
    ///
    /// pin.set_high().unwrap();
    /// let mut w5500: W5500<_, _> = W5500::new(spi, pin);
    /// # Ok::<(), hal::MockError>(())
    /// ```
    pub fn new(spi: SPI, cs: CS) -> Self {
        W5500 { spi, cs }
    }

    /// Free the SPI bus and CS pin from the W5500.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[]);
    /// # struct Pin {};
    /// # impl embedded_hal::digital::v2::OutputPin for Pin {
    /// #     type Error = core::convert::Infallible;
    /// #     fn set_low(&mut self) -> Result<(), Self::Error> { Ok(()) }
    /// #     fn set_high(&mut self) -> Result<(), Self::Error> { Ok(()) }
    /// # }
    /// # let mut pin = Pin {};
    /// use w5500_ll::blocking::vdm_infallible_gpio::W5500;
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// let (spi, pin) = w5500.free();
    /// ```
    pub fn free(self) -> (SPI, CS) {
        (self.spi, self.cs)
    }

    #[inline(always)]
    fn with_chip_enable<T, F>(&mut self, mut f: F) -> Result<T, SpiError>
    where
        F: FnMut(&mut SPI) -> Result<T, SpiError>,
    {
        self.cs.set_low().unwrap();
        let result = f(&mut self.spi);
        self.cs.set_high().unwrap();
        result
    }
}

impl<SPI, CS, SpiError> crate::Registers for W5500<SPI, CS>
where
    SPI: embedded_hal::blocking::spi::Transfer<u8, Error = SpiError>
        + embedded_hal::blocking::spi::Write<u8, Error = SpiError>,
    CS: OutputPin<Error = core::convert::Infallible>,
{
    /// SPI IO error type.
    type Error = SpiError;

    /// Read from the W5500.
    #[inline(always)]
    fn read(&mut self, address: u16, block: u8, data: &mut [u8]) -> Result<(), Self::Error> {
        let header = vdm_header(address, block, AccessMode::Read);
        self.with_chip_enable(|spi| {
            spi.write(&header)?;
            spi.transfer(data)?;
            Ok(())
        })
    }

    /// Write to the W5500.
    #[inline(always)]
    fn write(&mut self, address: u16, block: u8, data: &[u8]) -> Result<(), Self::Error> {
        let header = vdm_header(address, block, AccessMode::Write);
        self.with_chip_enable(|spi| {
            spi.write(&header)?;
            spi.write(data)?;
            Ok(())
        })
    }
}
