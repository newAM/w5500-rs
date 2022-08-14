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
use embedded_hal::spi::blocking::{SpiBusRead, SpiBusWrite};

/// W5500 blocking variable data length implementation.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct W5500<SPI> {
    /// SPI bus + chip select pin.
    spi: SPI,
}

impl<SPI, E> W5500<SPI>
where
    SPI: embedded_hal::spi::blocking::SpiDevice<Error = E>,
{
    /// Creates a new `W5500` driver from a SPI device.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[]);
    /// use w5500_ll::eh::vdm::W5500;
    ///
    /// let mut w5500: W5500<_> = W5500::new(spi);
    /// ```
    pub fn new(spi: SPI) -> Self {
        W5500 { spi }
    }

    /// Free the SPI device.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[]);
    /// use w5500_ll::eh::vdm::W5500;
    ///
    /// let mut w5500 = W5500::new(spi);
    /// let spi = w5500.free();
    /// ```
    pub fn free(self) -> SPI {
        self.spi
    }
}

impl<SPI, E> crate::Registers for W5500<SPI>
where
    SPI: embedded_hal::spi::blocking::SpiDevice<Error = E>,
    SPI::Bus: embedded_hal::spi::blocking::SpiBusRead<Error = E>
        + embedded_hal::spi::blocking::SpiBusWrite<Error = E>,
{
    /// SPI IO error type.
    type Error = E;

    /// Read from the W5500.
    #[inline]
    fn read(&mut self, address: u16, block: u8, data: &mut [u8]) -> Result<(), Self::Error> {
        let header = vdm_header(address, block, AccessMode::Read);
        self.spi.transaction(|bus| {
            bus.write(&header)?;
            bus.read(data)
        })
    }

    /// Write to the W5500.
    #[inline]
    fn write(&mut self, address: u16, block: u8, data: &[u8]) -> Result<(), Self::Error> {
        let header = vdm_header(address, block, AccessMode::Write);
        self.spi.transaction(|bus| {
            bus.write(&header)?;
            bus.write(data)
        })
    }
}