//! Implementation of the W5500 [`crate::Registers`] trait using the
//! [`embedded-hal`] blocking SPI traits.
//!
//! [`embedded-hal`]: https://github.com/rust-embedded/embedded-hal

use embedded_hal::digital::v2::OutputPin;

/// SPI Access Modes.
#[repr(u8)]
enum AccessMode {
    /// Read access.
    Read = 0,
    /// Write access.
    Write = 1,
}
impl From<AccessMode> for u8 {
    fn from(val: AccessMode) -> Self {
        val as u8
    }
}

/// Helper to generate a SPI header.
#[inline(always)]
const fn spi_header(address: u16, block: u8, mode: AccessMode) -> [u8; 3] {
    [
        (address >> 8) as u8,
        address as u8,
        (block << 3) | ((mode as u8) << 2),
    ]
}

/// W5500 blocking implementation.
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
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[]);
    /// # let pin = hal::pin::Mock::new(&[]);
    /// use w5500_ll::blocking::W5500;
    ///
    /// let mut w5500: W5500<_, _> = W5500::new(spi, pin);
    /// ```
    pub fn new(spi: SPI, cs: CS) -> Self {
        W5500 { spi, cs }
    }

    /// Free the SPI bus and CS pin from the W5500.
    ///
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[]);
    /// # let pin = hal::pin::Mock::new(&[]);
    /// use w5500_ll::blocking::W5500;
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
        let header = spi_header(address, block, AccessMode::Read);
        self.with_chip_enable(|spi| {
            spi.write(&header).map_err(Error::Spi)?;
            spi.transfer(data).map_err(Error::Spi)?;
            Ok(())
        })
    }

    /// Write to the W5500.
    #[inline(always)]
    fn write(&mut self, address: u16, block: u8, data: &[u8]) -> Result<(), Self::Error> {
        let header = spi_header(address, block, AccessMode::Write);
        self.with_chip_enable(|spi| {
            spi.write(&header).map_err(Error::Spi)?;
            spi.write(&data).map_err(Error::Spi)?;
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Socket;

    macro_rules! spi_header_tests {
        ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let ((address, block, mode), expected) = $value;
                assert_eq!(spi_header(address, block, mode), expected);
            }
        )*
        }
    }

    spi_header_tests! {
        spi_header_0: ((0, 0, AccessMode::Read), [0, 0, 0]),
        spi_header_1: ((0x1234, 0, AccessMode::Read), [0x12, 0x34, 0]),
        spi_header_2: ((0, Socket::Socket0.block(), AccessMode::Read), [0, 0, 8]),
        spi_header_3: ((0, Socket::Socket7.tx_block(), AccessMode::Read), [0, 0, 0b11110 << 3]),
        spi_header_4: ((0, Socket::Socket7.rx_block(), AccessMode::Read), [0, 0, 0b11111 << 3]),
        spi_header_5: ((0, 0, AccessMode::Write), [0, 0, 4]),
    }
}
