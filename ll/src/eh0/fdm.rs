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
//! [VDM]: crate::eh0::vdm

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
    SPI: eh0::blocking::spi::Transfer<u8, Error = SpiError>
        + eh0::blocking::spi::Write<u8, Error = SpiError>,
{
    /// Creates a new `W5500` driver from a SPI peripheral.
    ///
    /// # Example
    ///
    /// ```
    /// # use ehm::eh0 as hal;
    /// # let spi = hal::spi::Mock::new(&[]);
    /// use w5500_ll::eh0::fdm::W5500;
    ///
    /// let mut w5500: W5500<_> = W5500::new(spi);
    /// # w5500.free().done();
    /// ```
    #[inline]
    pub fn new(spi: SPI) -> Self {
        W5500 { spi }
    }

    /// Free the SPI bus from the W5500.
    ///
    /// # Example
    ///
    /// ```
    /// # use ehm::eh0 as hal;
    /// # let spi = hal::spi::Mock::new(&[]);
    /// use w5500_ll::eh0::fdm::W5500;
    ///
    /// let w5500: W5500<_> = W5500::new(spi);
    /// let mut spi = w5500.free();
    /// # spi.done();
    /// ```
    #[inline]
    pub fn free(self) -> SPI {
        self.spi
    }
}

impl<SPI, SpiError> crate::Registers for W5500<SPI>
where
    SPI: eh0::blocking::spi::Transfer<u8, Error = SpiError>
        + eh0::blocking::spi::Write<u8, Error = SpiError>,
{
    /// SPI IO error type.
    type Error = SpiError;

    /// Read from the W5500.
    fn read(&mut self, mut address: u16, block: u8, data: &mut [u8]) -> Result<(), Self::Error> {
        let (chunks4, rest) = data.as_chunks_mut::<4>();
        for chunk in chunks4 {
            let header = spi::fdm_header_4b(address, block, AccessMode::Read);
            self.spi.write(&header)?;
            self.spi.transfer(chunk)?;
            address = address.wrapping_add(4);
        }
        let (chunks2, rest) = rest.as_chunks_mut::<2>();
        for chunk in chunks2 {
            let header = spi::fdm_header_2b(address, block, AccessMode::Read);
            self.spi.write(&header)?;
            self.spi.transfer(chunk)?;
            address = address.wrapping_add(2);
        }
        let (chunks1, _rest) = rest.as_chunks_mut::<1>();
        for chunk in chunks1 {
            let header = spi::fdm_header_1b(address, block, AccessMode::Read);
            self.spi.write(&header)?;
            self.spi.transfer(chunk)?;
            address = address.wrapping_add(1);
        }

        Ok(())
    }

    /// Write to the W5500.
    fn write(&mut self, mut address: u16, block: u8, data: &[u8]) -> Result<(), Self::Error> {
        let (chunks4, rest) = data.as_chunks::<4>();
        for chunk in chunks4 {
            let header = spi::fdm_header_4b(address, block, AccessMode::Write);
            self.spi.write(&header)?;
            self.spi.write(chunk)?;
            address = address.wrapping_add(4);
        }
        let (chunks2, rest) = rest.as_chunks::<2>();
        for chunk in chunks2 {
            let header = spi::fdm_header_2b(address, block, AccessMode::Write);
            self.spi.write(&header)?;
            self.spi.write(chunk)?;
            address = address.wrapping_add(2);
        }
        let (chunks1, _rest) = rest.as_chunks::<1>();
        for chunk in chunks1 {
            let header = spi::fdm_header_1b(address, block, AccessMode::Write);
            self.spi.write(&header)?;
            self.spi.write(chunk)?;
            address = address.wrapping_add(1);
        }

        Ok(())
    }
}
