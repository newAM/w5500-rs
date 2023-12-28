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
use eh1::spi::ErrorType;

/// W5500 blocking variable data length implementation.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct W5500<SPI> {
    /// SPI bus + chip select pin.
    spi: SPI,
}

impl<SPI, E> W5500<SPI>
where
    SPI: eh1::spi::SpiDevice<Error = E>,
{
    /// Creates a new `W5500` driver from a SPI device.
    ///
    /// # Example
    ///
    /// ```
    /// # use ehm::eh1 as hal;
    /// # let spi = hal::spi::Mock::new(&[]);
    /// use w5500_ll::eh1::vdm::W5500;
    ///
    /// let mut w5500: W5500<_> = W5500::new(spi);
    /// # w5500.free().done();
    /// ```
    #[inline]
    pub fn new(spi: SPI) -> Self {
        W5500 { spi }
    }

    /// Free the SPI device.
    ///
    /// # Example
    ///
    /// ```
    /// # use ehm::eh1 as hal;
    /// # let spi = hal::spi::Mock::new(&[]);
    /// use w5500_ll::eh1::vdm::W5500;
    ///
    /// let mut w5500 = W5500::new(spi);
    /// let mut spi = w5500.free();
    /// # spi.done();
    /// ```
    #[inline]
    pub fn free(self) -> SPI {
        self.spi
    }
}

impl<SPI> crate::Registers for W5500<SPI>
where
    SPI: eh1::spi::SpiDevice,
{
    /// SPI IO error type.
    type Error = SPI::Error;

    /// Read from the W5500.
    #[inline]
    fn read(
        &mut self,
        address: u16,
        block: u8,
        data: &mut [u8],
    ) -> Result<(), <SPI as ErrorType>::Error> {
        let header = vdm_header(address, block, AccessMode::Read);
        let mut ops = [
            eh1::spi::Operation::Write(&header),
            eh1::spi::Operation::Read(data),
        ];
        self.spi.transaction(&mut ops)
    }

    /// Write to the W5500.
    #[inline]
    fn write(
        &mut self,
        address: u16,
        block: u8,
        data: &[u8],
    ) -> Result<(), <SPI as ErrorType>::Error> {
        let header = vdm_header(address, block, AccessMode::Write);
        let mut ops = [
            eh1::spi::Operation::Write(&header),
            eh1::spi::Operation::Write(data),
        ];
        self.spi.transaction(&mut ops)
    }
}

#[cfg(feature = "eha1")]
impl<SPI> crate::aio::Registers for W5500<SPI>
where
    SPI: eha1::spi::SpiDevice,
{
    /// SPI IO error type.
    type Error = SPI::Error;

    /// Read from the W5500 asynchronously.
    async fn read(
        &mut self,
        address: u16,
        block: u8,
        data: &mut [u8],
    ) -> Result<(), <SPI as ErrorType>::Error> {
        let header = vdm_header(address, block, AccessMode::Read);
        let mut ops = [
            eh1::spi::Operation::Write(&header),
            eh1::spi::Operation::Read(data),
        ];
        self.spi.transaction(&mut ops).await
    }

    /// Write to the W5500 asynchronously.
    async fn write(
        &mut self,
        address: u16,
        block: u8,
        data: &[u8],
    ) -> Result<(), <SPI as ErrorType>::Error> {
        let header = vdm_header(address, block, AccessMode::Write);
        let mut ops = [
            eh1::spi::Operation::Write(&header),
            eh1::spi::Operation::Write(data),
        ];
        self.spi.transaction(&mut ops).await
    }
}
