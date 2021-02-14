//! Variable data length implementation of the [`Registers`] trait using the
//! [`embedded-hal`] blocking SPI trait.
//!
//! This uses the W5500 variable data length mode (VDM).
//! In VDM mode the SPI frame data length is determined by the chip select pin.
//! This is the preferred blocking implementation if your W5500 has a chip
//! select pin.
//!
//! [`embedded-hal`]: https://github.com/rust-embedded/embedded-hal
//! [`Registers`]: crate::Registers

use crate::spi::{vdm_header, AccessMode};
use embedded_hal::digital::v2::OutputPin;

/// W5500 blocking variable data length implementation.
pub struct W5500<SPI, CS> {
    /// SPI bus.
    spi: SPI,
    /// GPIO for chip select.
    cs: CS,
}

/// W5500 blocking implementation error type.
#[derive(Debug)]
pub enum Error<SpiError, PinError> {
    /// SPI bus error wrapper.
    Spi(SpiError),
    /// GPIO pin error wrapper.
    Pin(PinError),
}

impl<SPI, CS, SpiError, PinError> W5500<SPI, CS>
where
    SPI: embedded_hal::blocking::spi::Transfer<u8, Error = SpiError>
        + embedded_hal::blocking::spi::Write<u8, Error = SpiError>,
    CS: OutputPin<Error = PinError>,
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
    /// # let mut pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use embedded_hal::digital::v2::OutputPin;
    /// use w5500_ll::blocking::vdm::W5500;
    ///
    /// pin.set_high()?;
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
    /// # let pin = hal::pin::Mock::new(&[]);
    /// use w5500_ll::blocking::vdm::W5500;
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// let (spi, pin) = w5500.free();
    /// ```
    pub fn free(self) -> (SPI, CS) {
        (self.spi, self.cs)
    }

    #[inline(always)]
    fn with_chip_enable<T, E, F>(&mut self, mut f: F) -> Result<T, E>
    where
        F: FnMut(&mut SPI) -> Result<T, E>,
        E: core::convert::From<Error<SpiError, PinError>>,
    {
        self.cs.set_low().map_err(Error::Pin)?;
        let result = f(&mut self.spi);
        self.cs.set_high().map_err(Error::Pin)?;
        result
    }
}

impl<SPI, CS, SpiError, PinError> crate::Registers for W5500<SPI, CS>
where
    SPI: embedded_hal::blocking::spi::Transfer<u8, Error = SpiError>
        + embedded_hal::blocking::spi::Write<u8, Error = SpiError>,
    CS: OutputPin<Error = PinError>,
{
    /// SPI IO error type.
    type Error = Error<SpiError, PinError>;

    /// Read from the W5500.
    #[inline(always)]
    fn read(&mut self, address: u16, block: u8, data: &mut [u8]) -> Result<(), Self::Error> {
        let header = vdm_header(address, block, AccessMode::Read);
        self.with_chip_enable(|spi| {
            spi.write(&header).map_err(Error::Spi)?;
            spi.transfer(data).map_err(Error::Spi)?;
            Ok(())
        })
    }

    /// Write to the W5500.
    #[inline(always)]
    fn write(&mut self, address: u16, block: u8, data: &[u8]) -> Result<(), Self::Error> {
        let header = vdm_header(address, block, AccessMode::Write);
        self.with_chip_enable(|spi| {
            spi.write(&header).map_err(Error::Spi)?;
            spi.write(&data).map_err(Error::Spi)?;
            Ok(())
        })
    }
}
