//! Fixed data length implementation of the [`Registers`] trait using the
//! [`embedded-hal`] blocking SPI trait.
//!
//! This uses the W5500 fixed data length mode (FDM).
//! In FSM mode the SPI chip select pin is always tied low, and it is not
//! possible to share the bus with other devices.
//!
//! If possible, you should use the [VDM] implementation instead.
//!
//! [`embedded-hal`]: https://github.com/rust-embedded/embedded-hal
//! [`Registers`]: crate::Registers
//! [VDM]: crate::blocking::vdm

use crate::spi::{self, AccessMode};

/// W5500 blocking fixed data length implementation.
///
/// Unlike the VDM implementation there is an intentional lack of a `free`
/// method to prevent you from sharing the bus with other devices.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct W5500<SPI> {
    /// SPI bus.
    spi: SPI,
}

impl<SPI, SpiError> W5500<SPI>
where
    SPI: embedded_hal::blocking::spi::Transfer<u8, Error = SpiError>
        + embedded_hal::blocking::spi::Write<u8, Error = SpiError>,
{
    /// Creates a new `W5500` driver from a SPI peripheral.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[]);
    /// use w5500_ll::blocking::fdm::W5500;
    ///
    /// let mut w5500: W5500<_> = W5500::new(spi);
    /// ```
    pub fn new(spi: SPI) -> Self {
        W5500 { spi }
    }

    /// Free the SPI bus from the W5500.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[]);
    /// use w5500_ll::blocking::fdm::W5500;
    ///
    /// let w5500: W5500<_> = W5500::new(spi);
    /// let spi = w5500.free();
    /// ```
    pub fn free(self) -> SPI {
        self.spi
    }
}

impl<SPI, SpiError> crate::Registers for W5500<SPI>
where
    SPI: embedded_hal::blocking::spi::Transfer<u8, Error = SpiError>
        + embedded_hal::blocking::spi::Write<u8, Error = SpiError>,
{
    /// SPI IO error type.
    type Error = SpiError;

    /// Read from the W5500.
    #[allow(clippy::while_let_on_iterator)]
    fn read(&mut self, mut address: u16, block: u8, data: &mut [u8]) -> Result<(), Self::Error> {
        let mut chunks = data.chunks_exact_mut(4);
        while let Some(chunk) = chunks.next() {
            let header = spi::fdm_header_4b(address, block, AccessMode::Read);
            self.spi.write(&header)?;
            self.spi.transfer(chunk)?;
            address = address.wrapping_add(4);
        }
        let mut chunks = chunks.into_remainder().chunks_exact_mut(2);
        while let Some(chunk) = chunks.next() {
            let header = spi::fdm_header_2b(address, block, AccessMode::Read);
            self.spi.write(&header)?;
            self.spi.transfer(chunk)?;
            address = address.wrapping_add(2);
        }
        let mut chunks = chunks.into_remainder().chunks_exact_mut(1);
        while let Some(chunk) = chunks.next() {
            let header = spi::fdm_header_1b(address, block, AccessMode::Read);
            self.spi.write(&header)?;
            self.spi.transfer(chunk)?;
            address = address.wrapping_add(1);
        }

        Ok(())
    }

    /// Write to the W5500.
    #[allow(clippy::while_let_on_iterator)]
    fn write(&mut self, mut address: u16, block: u8, data: &[u8]) -> Result<(), Self::Error> {
        let mut chunks = data.chunks_exact(4);
        while let Some(chunk) = chunks.next() {
            let header = spi::fdm_header_4b(address, block, AccessMode::Write);
            self.spi.write(&header)?;
            self.spi.write(&chunk)?;
            address = address.wrapping_add(4);
        }
        let mut chunks = chunks.remainder().chunks_exact(2);
        while let Some(chunk) = chunks.next() {
            let header = spi::fdm_header_2b(address, block, AccessMode::Write);
            self.spi.write(&header)?;
            self.spi.write(&chunk)?;
            address = address.wrapping_add(2);
        }
        let mut chunks = chunks.remainder().chunks_exact(1);
        while let Some(chunk) = chunks.next() {
            let header = spi::fdm_header_1b(address, block, AccessMode::Write);
            self.spi.write(&header)?;
            self.spi.write(&chunk)?;
            address = address.wrapping_add(1);
        }

        Ok(())
    }
}
