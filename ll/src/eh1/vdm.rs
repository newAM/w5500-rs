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
use eh1::spi::{SpiBusRead, SpiBusWrite};

#[cfg(feature = "eha0")]
use eha0::spi::{SpiBusRead as AioSpiBusRead, SpiBusWrite as AioSpiBusWrite};

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
    /// # use ehm1 as hal;
    /// # let spi = hal::spi::Mock::new(&[]);
    /// use w5500_ll::eh1::vdm::W5500;
    ///
    /// let mut w5500: W5500<_> = W5500::new(spi);
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
    /// # use ehm1 as hal;
    /// # let spi = hal::spi::Mock::new(&[]);
    /// use w5500_ll::eh1::vdm::W5500;
    ///
    /// let mut w5500 = W5500::new(spi);
    /// let spi = w5500.free();
    /// ```
    #[inline]
    pub fn free(self) -> SPI {
        self.spi
    }
}

impl<SPI, E> crate::Registers for W5500<SPI>
where
    SPI: eh1::spi::SpiDevice<Error = E>,
    SPI::Bus: eh1::spi::SpiBusRead<Error = E> + eh1::spi::SpiBusWrite<Error = E>,
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

#[cfg(feature = "eha0")]
#[allow(unsafe_code)]
impl<SPI, E> crate::aio::Registers for W5500<SPI>
where
    SPI: eha0::spi::SpiDevice<Error = E>,

    <SPI as eha0::spi::SpiDevice>::Bus:
        eha0::spi::SpiBusRead<Error = E> + eha0::spi::SpiBusWrite<Error = E>,
{
    /// SPI IO error type.
    type Error = E;

    /// Read from the W5500 asynchronously.
    async fn read(&mut self, address: u16, block: u8, data: &mut [u8]) -> Result<(), Self::Error> {
        let header = vdm_header(address, block, AccessMode::Read);
        eha0::spi::SpiDevice::transaction(&mut self.spi, move |bus| async move {
            let bus = unsafe { &mut *bus };
            bus.write(&header).await?;
            bus.read(data).await
        })
        .await
    }

    /// Write to the W5500 asynchronously.
    async fn write(&mut self, address: u16, block: u8, data: &[u8]) -> Result<(), Self::Error> {
        let header = vdm_header(address, block, AccessMode::Write);
        eha0::spi::SpiDevice::transaction(&mut self.spi, move |bus| async move {
            let bus = unsafe { &mut *bus };
            bus.write(&header).await?;
            bus.write(data).await
        })
        .await
    }
}
