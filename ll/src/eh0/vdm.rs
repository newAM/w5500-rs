//! Variable data length implementation of the [`Registers`] trait using the
//! [`embedded-hal`] blocking SPI trait, and a fallible GPIO pin.
//!
//! This uses the W5500 variable data length mode (VDM).
//! In VDM mode the SPI frame data length is determined by the chip select pin.
//! This is the preferred blocking implementation if your W5500 has a fallible
//! chip select pin.
//!
//! [`embedded-hal`]: https://github.com/rust-embedded/embedded-hal
//! [`Registers`]: crate::Registers

use crate::spi::{vdm_header, AccessMode};
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

/// W5500 blocking implementation error type.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error<SpiError, PinError> {
    /// SPI bus error wrapper.
    Spi(SpiError),
    /// GPIO pin error wrapper.
    Pin(PinError),
}

impl<SPI, CS, SpiError, PinError> W5500<SPI, CS>
where
    SPI: eh0::blocking::spi::Transfer<u8, Error = SpiError>
        + eh0::blocking::spi::Write<u8, Error = SpiError>,
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
    /// # use ehm0 as hal;
    /// # let spi = hal::spi::Mock::new(&[]);
    /// # let mut pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use eh0::digital::v2::OutputPin;
    /// use w5500_ll::eh0::vdm::W5500;
    ///
    /// pin.set_high()?;
    /// let mut w5500: W5500<_, _> = W5500::new(spi, pin);
    /// # Ok::<(), hal::MockError>(())
    /// ```
    #[inline]
    pub fn new(spi: SPI, cs: CS) -> Self {
        W5500 { spi, cs }
    }

    /// Free the SPI bus and CS pin from the W5500.
    ///
    /// # Example
    ///
    /// ```
    /// # use ehm0 as hal;
    /// # let spi = hal::spi::Mock::new(&[]);
    /// # let pin = hal::pin::Mock::new(&[]);
    /// use w5500_ll::eh0::vdm::W5500;
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// let (spi, pin) = w5500.free();
    /// ```
    #[inline]
    pub fn free(self) -> (SPI, CS) {
        (self.spi, self.cs)
    }

    #[inline]
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
    SPI: eh0::blocking::spi::Transfer<u8, Error = SpiError>
        + eh0::blocking::spi::Write<u8, Error = SpiError>,
    CS: OutputPin<Error = PinError>,
{
    /// SPI IO error type.
    type Error = Error<SpiError, PinError>;

    /// Read from the W5500.
    #[inline]
    fn read(&mut self, address: u16, block: u8, data: &mut [u8]) -> Result<(), Self::Error> {
        let header = vdm_header(address, block, AccessMode::Read);
        self.with_chip_enable(|spi| {
            spi.write(&header).map_err(Error::Spi)?;
            spi.transfer(data).map_err(Error::Spi)?;
            Ok(())
        })
    }

    /// Write to the W5500.
    #[inline]
    fn write(&mut self, address: u16, block: u8, data: &[u8]) -> Result<(), Self::Error> {
        let header = vdm_header(address, block, AccessMode::Write);
        self.with_chip_enable(|spi| {
            spi.write(&header).map_err(Error::Spi)?;
            spi.write(data).map_err(Error::Spi)?;
            Ok(())
        })
    }
}
