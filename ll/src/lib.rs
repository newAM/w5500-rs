//! Platform agnostic rust driver for the [Wiznet W5500] SPI internet offload
//! chip.
//!
//! This is a low-level (ll) crate, specifically limited in scope to register
//! accessors only.
//! Higher level functionality (such as socket operations) should be built
//! on-top of what is provided here.
//!
//! [Wiznet W5500]: https://www.wiznet.io/product-item/w5500/
#![doc(html_root_url = "https://docs.rs/w5500-ll/0.2.0")]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![no_std]

pub mod blocking;
pub mod net;

mod registers;
mod specifiers;
use core::convert::TryFrom;
use embedded_hal::spi::{self, Phase, Polarity};
use net::{Eui48Addr, Ipv4Addr};

pub use registers::{Interrupt, Mode, PhyCfg, SocketInterrupt, SocketInterruptMask, SocketMode};
pub use specifiers::{
    BufferSize, DuplexStatus, LinkStatus, OperationMode, Protocol, SocketCommand, SocketStatus,
    SpeedStatus,
};

/// Socket spacing between blocks.
const SOCKET_SPACING: u8 = 0x04;
/// Common register block address offset.
const COMMON_BLOCK_OFFSET: u8 = 0x00;
/// Socket common block select bits offset.
const SOCKET_BLOCK_OFFSET: u8 = 0x01;
/// Socket TX block select bits offset
const SOCKET_TX_OFFSET: u8 = 0x02;
/// Socket RX block select bits offset
const SOCKET_RX_OFFSET: u8 = 0x03;
/// Value of the W5500 VERSIONR register.
pub const VERSION: u8 = 0x04;

/// W5500 register addresses.
pub mod reg {
    /// Address of the MR register.
    pub const MR: u16 = 0x0000;
    /// Address of the GAR register.
    pub const GAR: u16 = 0x0001;
    /// Address of the SUBR register.
    pub const SUBR: u16 = 0x0005;
    /// Address of the SHAR register.
    pub const SHAR: u16 = 0x0009;
    /// Address of the SIPR register.
    pub const SIPR: u16 = 0x000F;
    /// Address of the INTLEVEL register.
    pub const INTLEVEL: u16 = 0x0013;
    /// Address of the IR register.
    pub const IR: u16 = 0x0015;
    /// Address of the IMR register.
    pub const IMR: u16 = 0x0016;
    /// Address of the SIR register.
    pub const SIR: u16 = 0x0017;
    /// Address of the SIMR register.
    pub const SIMR: u16 = 0x0018;
    /// Address of the RTR register.
    pub const RTR: u16 = 0x0019;
    /// Address of the RCR register.
    pub const RCR: u16 = 0x001B;
    /// Address of the PTIMER register.
    pub const PTIMER: u16 = 0x001C;
    /// Address of the PMAGIC register.
    pub const PMAGIC: u16 = 0x001D;
    /// Address of the PHAR register.
    pub const PHAR: u16 = 0x001E;
    /// Address of the PSID register.
    pub const PSID: u16 = 0x0024;
    /// Address of the PMRU register.
    pub const PMRU: u16 = 0x0026;
    /// Address of the UIPR register.
    pub const UIPR: u16 = 0x0028;
    /// Address of the UPORTR register.
    pub const UPORTR: u16 = 0x002C;
    /// Address of the PHYCFGR register.
    pub const PHYCFGR: u16 = 0x002E;
    /// Address of the VERSIONR register.
    pub const VERSIONR: u16 = 0x0039;

    /// Address of the SN_MR register.
    pub const SN_MR: u16 = 0x0000;
    /// Address of the SN_CR register.
    pub const SN_CR: u16 = 0x0001;
    /// Address of the SN_IR register.
    pub const SN_IR: u16 = 0x0002;
    /// Address of the SN_SR register.
    pub const SN_SR: u16 = 0x0003;
    /// Address of the SN_PORT register.
    pub const SN_PORT: u16 = 0x0004;
    /// Address of the SN_DHAR register.
    pub const SN_DHAR: u16 = 0x0006;
    /// Address of the SN_DIPR register.
    pub const SN_DIPR: u16 = 0x000C;
    /// Address of the SN_DPORT register.
    pub const SN_DPORT: u16 = 0x0010;
    /// Address of the SN_MSSR register.
    pub const SN_MSSR: u16 = 0x0012;
    /// Address of the SN_TOS register.
    pub const SN_TOS: u16 = 0x0015;
    /// Address of the SN_TTL register.
    pub const SN_TTL: u16 = 0x0016;
    /// Address of the SN_RXBUF_SIZE register.
    pub const SN_RXBUF_SIZE: u16 = 0x001E;
    /// Address of the SN_TXBUF_SIZE register.
    pub const SN_TXBUF_SIZE: u16 = 0x001F;
    /// Address of the SN_TX_FSR register.
    pub const SN_TX_FSR: u16 = 0x0020;
    /// Address of the SN_TX_RD register.
    pub const SN_TX_RD: u16 = 0x0022;
    /// Address of the SN_TX_WR register.
    pub const SN_TX_WR: u16 = 0x0024;
    /// Address of the SN_RX_RSR register.
    pub const SN_RX_RSR: u16 = 0x0026;
    /// Address of the SN_RX_RD register.
    pub const SN_RX_RD: u16 = 0x0028;
    /// Address of the SN_RX_WR register.
    pub const SN_RX_WR: u16 = 0x002A;
    /// Address of the SN_IMR register.
    pub const SN_IMR: u16 = 0x002C;
    /// Address of the SN_FRAG register.
    pub const SN_FRAG: u16 = 0x002D;
    /// Address of the SN_KPALVTR register.
    pub const SN_KPALVTR: u16 = 0x002F;
}

/// Recommended W5500 SPI mode.
///
/// The W5500 may operate in SPI mode 0 or SPI mode 3.
pub const SPI_MODE: spi::Mode = spi::Mode {
    polarity: Polarity::IdleLow,
    phase: Phase::CaptureOnFirstTransition,
};

/// W5500 sockets.
#[repr(u8)]
#[allow(missing_docs)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Debug)]
pub enum Socket {
    Socket0 = 0,
    Socket1 = 1,
    Socket2 = 2,
    Socket3 = 3,
    Socket4 = 4,
    Socket5 = 5,
    Socket6 = 6,
    Socket7 = 7,
}

impl Socket {
    /// Get the socket register block select bits.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::Socket;
    ///
    /// assert_eq!(Socket::Socket0.block(), 0b00001);
    /// assert_eq!(Socket::Socket1.block(), 0b00101);
    /// assert_eq!(Socket::Socket2.block(), 0b01001);
    /// assert_eq!(Socket::Socket3.block(), 0b01101);
    /// assert_eq!(Socket::Socket4.block(), 0b10001);
    /// assert_eq!(Socket::Socket5.block(), 0b10101);
    /// assert_eq!(Socket::Socket6.block(), 0b11001);
    /// assert_eq!(Socket::Socket7.block(), 0b11101);
    /// ```
    #[inline(always)]
    pub const fn block(self) -> u8 {
        SOCKET_SPACING * (self as u8) + SOCKET_BLOCK_OFFSET
    }

    /// Get the socket TX buffer block select bits.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::Socket;
    ///
    /// assert_eq!(Socket::Socket0.tx_block(), 0b00010);
    /// assert_eq!(Socket::Socket1.tx_block(), 0b00110);
    /// assert_eq!(Socket::Socket2.tx_block(), 0b01010);
    /// assert_eq!(Socket::Socket3.tx_block(), 0b01110);
    /// assert_eq!(Socket::Socket4.tx_block(), 0b10010);
    /// assert_eq!(Socket::Socket5.tx_block(), 0b10110);
    /// assert_eq!(Socket::Socket6.tx_block(), 0b11010);
    /// assert_eq!(Socket::Socket7.tx_block(), 0b11110);
    /// ```
    #[inline(always)]
    pub const fn tx_block(self) -> u8 {
        SOCKET_SPACING * (self as u8) + SOCKET_TX_OFFSET
    }

    /// Get the socket RX buffer block select bits.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::Socket;
    ///
    /// assert_eq!(Socket::Socket0.rx_block(), 0b00011);
    /// assert_eq!(Socket::Socket1.rx_block(), 0b00111);
    /// assert_eq!(Socket::Socket2.rx_block(), 0b01011);
    /// assert_eq!(Socket::Socket3.rx_block(), 0b01111);
    /// assert_eq!(Socket::Socket4.rx_block(), 0b10011);
    /// assert_eq!(Socket::Socket5.rx_block(), 0b10111);
    /// assert_eq!(Socket::Socket6.rx_block(), 0b11011);
    /// assert_eq!(Socket::Socket7.rx_block(), 0b11111);
    /// ```
    #[inline(always)]
    pub const fn rx_block(self) -> u8 {
        SOCKET_SPACING * (self as u8) + SOCKET_RX_OFFSET
    }
}

impl From<Socket> for u8 {
    fn from(s: Socket) -> Self {
        s as u8
    }
}

impl TryFrom<u8> for Socket {
    type Error = u8;
    fn try_from(val: u8) -> Result<Socket, u8> {
        match val {
            0 => Ok(Socket::Socket0),
            1 => Ok(Socket::Socket1),
            2 => Ok(Socket::Socket2),
            3 => Ok(Socket::Socket3),
            4 => Ok(Socket::Socket4),
            5 => Ok(Socket::Socket5),
            6 => Ok(Socket::Socket6),
            7 => Ok(Socket::Socket7),
            x => Err(x),
        }
    }
}

/// Array of all sockets.
///
/// Useful for iterating over sockets.
pub const SOCKETS: [Socket; 8] = [
    Socket::Socket0,
    Socket::Socket1,
    Socket::Socket2,
    Socket::Socket3,
    Socket::Socket4,
    Socket::Socket5,
    Socket::Socket6,
    Socket::Socket7,
];

/// W5500 register setters and getters.
///
/// * All register getters are simply the name of the register.
/// * All register setters are the name of the register prefixed with `set_`.
///
/// Most of the register documentation is taken from the data sheet.
pub trait Registers {
    /// Register accessor error type.
    type Error;

    /// Read from the W5500.
    fn read(&mut self, address: u16, block: u8, data: &mut [u8]) -> Result<(), Self::Error>;

    /// Write to the W5500.
    fn write(&mut self, address: u16, block: u8, data: &[u8]) -> Result<(), Self::Error>;

    /// Get the mode register.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x00, 0x00]),
    /// #   hal::spi::Transaction::transfer(vec![0], vec![0]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, Mode, Registers};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// let mode: Mode = w5500.mr()?;
    /// assert_eq!(mode, Mode::default());
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn mr(&mut self) -> Result<Mode, Self::Error> {
        let mut reg: [u8; 1] = [0];
        self.read(reg::MR, COMMON_BLOCK_OFFSET, &mut reg)?;
        Ok(Mode::from(reg[0]))
    }

    /// Set the mode register.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x00, 0x04]),
    /// #   hal::spi::Transaction::write(vec![w5500_ll::Mode::WOL_MASK]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, Mode, Registers};
    ///
    /// let mut mode: Mode = Mode::default();
    /// mode.enable_wol();
    /// let mut w5500 = W5500::new(spi, pin);
    /// w5500.set_mr(mode)?;
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn set_mr(&mut self, mode: Mode) -> Result<(), Self::Error> {
        self.write(reg::MR, COMMON_BLOCK_OFFSET, &[mode.into()])
    }

    /// Get the gateway IP address.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x01, 0x00]),
    /// #   hal::spi::Transaction::transfer(vec![0, 0, 0, 0], vec![0, 0, 0, 0]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, net::Ipv4Addr, Registers};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// let gar = w5500.gar()?;
    /// assert_eq!(gar, Ipv4Addr::UNSPECIFIED);
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn gar(&mut self) -> Result<Ipv4Addr, Self::Error> {
        let mut gar = Ipv4Addr::UNSPECIFIED;
        self.read(reg::GAR, COMMON_BLOCK_OFFSET, &mut gar.octets)?;
        Ok(gar)
    }

    /// Set the gateway IP address.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x01, 0x04]),
    /// #   hal::spi::Transaction::write(vec![192, 168, 0, 1]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, net::Ipv4Addr, Registers};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// w5500.set_gar(&Ipv4Addr::new(192, 168, 0, 1))?;
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn set_gar(&mut self, gar: &Ipv4Addr) -> Result<(), Self::Error> {
        self.write(reg::GAR, COMMON_BLOCK_OFFSET, &gar.octets)
    }

    /// Get the subnet mask.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x05, 0x00]),
    /// #   hal::spi::Transaction::transfer(vec![0, 0, 0, 0], vec![0, 0, 0, 0]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, net::Ipv4Addr, Registers};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// let subr = w5500.subr()?;
    /// assert_eq!(subr, Ipv4Addr::UNSPECIFIED);
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn subr(&mut self) -> Result<Ipv4Addr, Self::Error> {
        let mut subr = Ipv4Addr::UNSPECIFIED;
        self.read(reg::SUBR, COMMON_BLOCK_OFFSET, &mut subr.octets)?;
        Ok(subr)
    }

    /// Set the subnet mask.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x05, 0x04]),
    /// #   hal::spi::Transaction::write(vec![255, 255, 255, 0]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, net::Ipv4Addr, Registers};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// w5500.set_subr(&Ipv4Addr::new(255, 255, 255, 0))?;
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn set_subr(&mut self, subr: &Ipv4Addr) -> Result<(), Self::Error> {
        self.write(reg::SUBR, COMMON_BLOCK_OFFSET, &subr.octets)
    }

    /// Get the source hardware address.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x09, 0x00]),
    /// #   hal::spi::Transaction::transfer(vec![0, 0, 0, 0, 0, 0], vec![0, 0, 0, 0, 0, 0]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, net::Eui48Addr, Registers};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// let shar = w5500.shar()?;
    /// assert_eq!(shar, Eui48Addr::UNSPECIFIED);
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn shar(&mut self) -> Result<Eui48Addr, Self::Error> {
        let mut shar = Eui48Addr::UNSPECIFIED;
        self.read(reg::SHAR, COMMON_BLOCK_OFFSET, &mut shar.octets)?;
        Ok(shar)
    }

    /// Set the source hardware address.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x09, 0x04]),
    /// #   hal::spi::Transaction::write(vec![0x12, 0x34, 0, 0, 0, 0]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, net::Eui48Addr, Registers};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// w5500.set_shar(&Eui48Addr::new(0x12, 0x34, 0x00, 0x00, 0x00, 0x00))?;
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn set_shar(&mut self, shar: &Eui48Addr) -> Result<(), Self::Error> {
        self.write(reg::SHAR, COMMON_BLOCK_OFFSET, &shar.octets)
    }

    /// Get the source (client) IP address.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x0F, 0x00]),
    /// #   hal::spi::Transaction::transfer(vec![0, 0, 0, 0], vec![0, 0, 0, 0]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, net::Ipv4Addr, Registers};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// let sipr = w5500.sipr()?;
    /// assert_eq!(sipr, Ipv4Addr::UNSPECIFIED);
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn sipr(&mut self) -> Result<Ipv4Addr, Self::Error> {
        let mut sipr = Ipv4Addr::UNSPECIFIED;
        self.read(reg::SIPR, COMMON_BLOCK_OFFSET, &mut sipr.octets)?;
        Ok(sipr)
    }

    /// Set the source (client) IP address.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x0F, 0x04]),
    /// #   hal::spi::Transaction::write(vec![192, 168, 0, 150]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, net::Ipv4Addr, Registers};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// w5500.set_sipr(&Ipv4Addr::new(192, 168, 0, 150))?;
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn set_sipr(&mut self, sipr: &Ipv4Addr) -> Result<(), Self::Error> {
        self.write(reg::SIPR, COMMON_BLOCK_OFFSET, &sipr.octets)
    }

    /// Get the interrupt low level time.
    ///
    /// INTLEVEL configures the assert wait time (I<sub>AWT</sub>).
    ///
    /// When the  next interrupt occurs, the interrupt in (INTn) will assert
    /// to low after INTLEVEL time.
    ///
    /// The equation is:
    ///
    /// I<sub>AWT</sub> = (INTLEVEL + 1) * PLL<sub>CLK</sub> * 4
    ///
    /// When INTLEVEL > 0.
    ///
    /// You might want to take a look at the data sheet, there is a handy
    /// timing diagram there.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x13, 0x00]),
    /// #   hal::spi::Transaction::transfer(vec![0, 0], vec![0, 0]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, Registers};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// let intlevel: u16 = w5500.intlevel()?;
    /// assert_eq!(intlevel, 0x00);
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn intlevel(&mut self) -> Result<u16, Self::Error> {
        let mut buf: [u8; 2] = [0; 2];
        self.read(reg::INTLEVEL, COMMON_BLOCK_OFFSET, &mut buf)?;
        Ok(u16::from_be_bytes(buf))
    }

    /// Set the interrupt low level time.
    ///
    /// See [`Registers::intlevel`] for more information.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x13, 0x04]),
    /// #   hal::spi::Transaction::write(vec![0x12, 0x34]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, Registers};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// w5500.set_intlevel(0x1234)?;
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn set_intlevel(&mut self, intlevel: u16) -> Result<(), Self::Error> {
        self.write(reg::INTLEVEL, COMMON_BLOCK_OFFSET, &intlevel.to_be_bytes())
    }

    /// Get the interrupt status.
    ///
    /// `1` indicates the interrupt is raised.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x15, 0x00]),
    /// #   hal::spi::Transaction::transfer(vec![0], vec![0]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, Interrupt, Registers};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// let ir: Interrupt = w5500.ir()?;
    /// assert_eq!(ir, Interrupt::default());
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn ir(&mut self) -> Result<Interrupt, Self::Error> {
        let mut reg: [u8; 1] = [0];
        self.read(reg::IR, COMMON_BLOCK_OFFSET, &mut reg)?;
        Ok(Interrupt::from(reg[0]))
    }

    /// Set the interrupt status.
    ///
    /// Setting an interrupt bit to `1` will clear the interrupt.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x15, 0x00]),
    /// #   hal::spi::Transaction::transfer(vec![0], vec![0]),
    /// #   hal::spi::Transaction::write(vec![0x00, 0x15, 0x04]),
    /// #   hal::spi::Transaction::write(vec![0]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, Interrupt, Registers};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// let ir: Interrupt = w5500.ir()?;
    /// w5500.set_ir(ir)?;
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn set_ir(&mut self, interrupt: Interrupt) -> Result<(), Self::Error> {
        self.write(reg::IR, COMMON_BLOCK_OFFSET, &[interrupt.into()])
    }

    /// Get the interrupt mask.
    ///
    /// `0` indicates the interrupt is masked.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x16, 0x00]),
    /// #   hal::spi::Transaction::transfer(vec![0], vec![0]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, Interrupt, Registers};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// let imr: Interrupt = w5500.imr()?;
    /// assert_eq!(imr, Interrupt::default());
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn imr(&mut self) -> Result<Interrupt, Self::Error> {
        let mut reg: [u8; 1] = [0];
        self.read(reg::IMR, COMMON_BLOCK_OFFSET, &mut reg)?;
        Ok(Interrupt::from(reg[0]))
    }

    /// Set the interrupt mask.
    ///
    /// Setting an interrupt bit to `1` will mask the interrupt.
    /// When a bit of IMR is `1` and the corresponding interrupt is `1` an
    /// interrupt will be issued.
    /// If a bit of IMR is `0`, and interrupt will not be issued even if the
    /// corresponding IR bit is `1`.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::{blocking::W5500, Interrupt, Registers};
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x16, 0x04]),
    /// #   hal::spi::Transaction::write(vec![Interrupt::MP_MASK]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    ///
    /// let mut imr: Interrupt = Interrupt::default();
    /// // enable the magic packet interrupt
    /// imr.set_mp();
    /// let mut w5500 = W5500::new(spi, pin);
    /// w5500.set_imr(imr)?;
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn set_imr(&mut self, mask: Interrupt) -> Result<(), Self::Error> {
        self.write(reg::IMR, COMMON_BLOCK_OFFSET, &[mask.into()])
    }

    /// Get the socket interrupt status.
    ///
    /// SIMR indicated the interrupt status of all sockets.
    /// Each bit of SIR will be `1` until [`Registers::sn_ir`] is cleared.
    /// If [`Registers::sn_ir`] is not equal to `0x00` the n<sub>th</sup>
    /// bit of `sir` is `1` and the INTn pin is asserted until SIR is `0x00`.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x17, 0x00]),
    /// #   hal::spi::Transaction::transfer(vec![0], vec![0]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, Registers, SOCKETS};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// let sir = w5500.sir()?;
    /// // clear all socket interrupts
    /// for socket in SOCKETS.iter() {
    ///     if 1 << (*socket as u8) & sir != 0 {
    ///         let sn_ir = w5500.sn_ir(*socket)?;
    ///         w5500.set_sn_ir(*socket, sn_ir)?;
    ///     }
    /// }
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn sir(&mut self) -> Result<u8, Self::Error> {
        let mut reg: [u8; 1] = [0];
        self.read(reg::SIR, COMMON_BLOCK_OFFSET, &mut reg)?;
        Ok(reg[0])
    }

    /// Get the socket interrupt mask.
    ///
    /// Each bit of SIMR corresponds to each bit of [`Registers::sir`].
    /// When a bit of SIMR is `1` and the corresponding bit of SIR is `1`
    /// and interrupt will be issued.
    /// If a bit of SIMR is `0` an interrupt will be not issued even if the
    /// corresponding bit of SIR is `1`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x18, 0x00]),
    /// #   hal::spi::Transaction::transfer(vec![0], vec![0]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, Registers};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// let simr: u8 = w5500.simr()?;
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn simr(&mut self) -> Result<u8, Self::Error> {
        let mut reg: [u8; 1] = [0];
        self.read(reg::SIMR, COMMON_BLOCK_OFFSET, &mut reg)?;
        Ok(reg[0])
    }

    /// Set the socket interrupt mask.
    ///
    /// See [`Registers::simr`] for more information.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x18, 0x00]),
    /// #   hal::spi::Transaction::write(vec![0xFF]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, Registers};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// // enable all socket interrupts
    /// w5500.set_simr(0xFF)?;
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn set_simr(&mut self, simr: u8) -> Result<(), Self::Error> {
        self.write(reg::SIMR, COMMON_BLOCK_OFFSET, &[simr])
    }

    /// Get the retry time.
    ///
    /// RTR configures the re-transmission timeout period.
    /// The unit of timeout period is 100us and the default of RTR is `0x07D0`
    /// or `2000`.
    /// And so the default timeout period is 200ms (100us X 2000).
    /// During the time configured by RTR, the W5500 waits for the peer response
    /// to the packet that is transmitted by Sn_CR (CONNECT, DISCON, CLOSE,
    /// SEND, SEND_MAC, SEND_KEEP command).
    /// If the peer does not respond within the RTR time, the W5500 re-transmits
    /// the packet or issues a timeout.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x19, 0x00]),
    /// #   hal::spi::Transaction::transfer(vec![0, 0], vec![0x07, 0xD0]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, Registers};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// let rtr: u16 = w5500.rtr()?;
    /// assert_eq!(rtr, 0x07D0);
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn rtr(&mut self) -> Result<u16, Self::Error> {
        let mut buf: [u8; 2] = [0; 2];
        self.read(reg::RTR, COMMON_BLOCK_OFFSET, &mut buf)?;
        Ok(u16::from_be_bytes(buf))
    }

    /// Set the retry time.
    ///
    /// See [`Registers::rtr`] for more information.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x19, 0x04]),
    /// #   hal::spi::Transaction::write(vec![0x12, 0x34]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, Registers};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// w5500.set_rtr(0x1234)?;
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn set_rtr(&mut self, rtr: u16) -> Result<(), Self::Error> {
        self.write(reg::RTR, COMMON_BLOCK_OFFSET, &rtr.to_be_bytes())
    }

    /// Get the retry count.
    ///
    /// RCR configured the number of re-transmission attempts.
    /// When the number of re-transmission equals RCR + 1 the socket timeout
    /// interrupt is raised.
    ///
    /// There is a LOT more information in the data sheet,
    /// including some equations that would be very annoying to input.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x1B, 0x00]),
    /// #   hal::spi::Transaction::transfer(vec![0], vec![0x08]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, Registers};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// let rcr: u8 = w5500.rcr()?;
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn rcr(&mut self) -> Result<u8, Self::Error> {
        let mut reg: [u8; 1] = [0];
        self.read(reg::RCR, COMMON_BLOCK_OFFSET, &mut reg)?;
        Ok(reg[0])
    }

    /// Set the retry count.
    ///
    /// See [`Registers::rcr`] for more information.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x1B, 0x04]),
    /// #   hal::spi::Transaction::write(vec![0x0A]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, Registers};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// w5500.set_rcr(0x0A)?;
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn set_rcr(&mut self, rcr: u8) -> Result<(), Self::Error> {
        self.write(reg::RCR, COMMON_BLOCK_OFFSET, &[rcr])
    }

    /// Get the PPP link control protocol request timer.
    ///
    /// PTIMER configures the time for sending LCP echo request.
    ///
    /// The unit of time is 25ms, for a register value of 200 the timer is 5
    /// seconds.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x1C, 0x00]),
    /// #   hal::spi::Transaction::transfer(vec![0], vec![0x08]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, Registers};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// let ptimer: u8 = w5500.ptimer()?;
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn ptimer(&mut self) -> Result<u8, Self::Error> {
        let mut reg: [u8; 1] = [0];
        self.read(reg::PTIMER, COMMON_BLOCK_OFFSET, &mut reg)?;
        Ok(reg[0])
    }

    /// Set the PPP link control protocol request timer.
    ///
    /// See [`Registers::ptimer`] for more information.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x1C, 0x04]),
    /// #   hal::spi::Transaction::write(vec![0xC8]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, Registers};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// w5500.set_ptimer(200)?;
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn set_ptimer(&mut self, ptimer: u8) -> Result<(), Self::Error> {
        self.write(reg::PTIMER, COMMON_BLOCK_OFFSET, &[ptimer])
    }

    /// Get the PPP link control protocol magic number.
    ///
    /// PMAGIC configures the 4 byte magic number used in the LCP echo request.
    /// For a register value of `0x01` the magic number is `0x01010101`.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x1D, 0x00]),
    /// #   hal::spi::Transaction::transfer(vec![0], vec![0x08]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, Registers};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// let pmagic: u8 = w5500.pmagic()?;
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn pmagic(&mut self) -> Result<u8, Self::Error> {
        let mut reg: [u8; 1] = [0];
        self.read(reg::PMAGIC, COMMON_BLOCK_OFFSET, &mut reg)?;
        Ok(reg[0])
    }

    /// Set the PPP link control protocol magic number.
    ///
    /// See [`Registers::pmagic`] for more information.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x1D, 0x04]),
    /// #   hal::spi::Transaction::write(vec![0x01]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, Registers};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// w5500.set_pmagic(0x01)?;
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn set_pmagic(&mut self, pmagic: u8) -> Result<(), Self::Error> {
        self.write(reg::PMAGIC, COMMON_BLOCK_OFFSET, &[pmagic])
    }

    /// Get the destination hardware address in PPPoE mode.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x1E, 0x00]),
    /// #   hal::spi::Transaction::transfer(vec![0, 0, 0, 0, 0, 0], vec![0, 0, 0, 0, 0, 0]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, net::Eui48Addr, Registers};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// let phar = w5500.phar()?;
    /// assert_eq!(phar, Eui48Addr::UNSPECIFIED);
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn phar(&mut self) -> Result<Eui48Addr, Self::Error> {
        let mut phar = Eui48Addr::UNSPECIFIED;
        self.read(reg::PHAR, COMMON_BLOCK_OFFSET, &mut phar.octets)?;
        Ok(phar)
    }

    /// Set the destination hardware address in PPPoE mode.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x1E, 0x04]),
    /// #   hal::spi::Transaction::write(vec![0x12, 0x34, 0, 0, 0, 0]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, net::Eui48Addr, Registers};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// w5500.set_phar(&Eui48Addr::new(0x12, 0x34, 0x00, 0x00, 0x00, 0x00))?;
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn set_phar(&mut self, phar: &Eui48Addr) -> Result<(), Self::Error> {
        self.write(reg::PHAR, COMMON_BLOCK_OFFSET, &phar.octets)
    }

    /// Get the session ID in PPPoE mode.
    ///
    /// PSID should be written to the PPPoE server session ID acquired in the
    /// PPPoE connection process.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x24, 0x00]),
    /// #   hal::spi::Transaction::transfer(vec![0, 0], vec![0, 0]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, Registers};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// let psid: u16 = w5500.psid()?;
    /// assert_eq!(psid, 0x0000);
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn psid(&mut self) -> Result<u16, Self::Error> {
        let mut buf: [u8; 2] = [0; 2];
        self.read(reg::PSID, COMMON_BLOCK_OFFSET, &mut buf)?;
        Ok(u16::from_be_bytes(buf))
    }

    /// Set the session ID in PPPoE mode.
    ///
    /// See [`Registers::psid`] for more information.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x24, 0x04]),
    /// #   hal::spi::Transaction::write(vec![0x12, 0x34]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, Registers};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// w5500.set_psid(0x1234)?;
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn set_psid(&mut self, psid: u16) -> Result<(), Self::Error> {
        self.write(reg::PSID, COMMON_BLOCK_OFFSET, &psid.to_be_bytes())
    }

    /// Get the maximum receive unit in PPPoE mode.
    ///
    /// PMRU configures the maximum receive unit of PPPoE.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x26, 0x00]),
    /// #   hal::spi::Transaction::transfer(vec![0, 0], vec![0, 0]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, Registers};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// let pmru: u16 = w5500.pmru()?;
    /// assert_eq!(pmru, 0x0000);
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn pmru(&mut self) -> Result<u16, Self::Error> {
        let mut buf: [u8; 2] = [0; 2];
        self.read(reg::PMRU, COMMON_BLOCK_OFFSET, &mut buf)?;
        Ok(u16::from_be_bytes(buf))
    }

    /// Set the maximum receive unit in PPPoE mode.
    ///
    /// See [`Registers::pmru`] for more information.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x26, 0x04]),
    /// #   hal::spi::Transaction::write(vec![0x12, 0x34]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, Registers};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// w5500.set_pmru(0x1234)?;
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn set_pmru(&mut self, pmru: u16) -> Result<(), Self::Error> {
        self.write(reg::PMRU, COMMON_BLOCK_OFFSET, &pmru.to_be_bytes())
    }

    /// Get the unreachable IP address.
    ///
    /// This awkward wording is taken directly from the data-sheet:
    ///
    /// W5500 receives an ICMP packet (destination port unreachable)
    /// when data is sent to a port number which socket is not open and
    /// the UNREACH bit of [`Registers::ir`] becomes `1` and UIPR and UPORTR
    /// indicates the destination IP address and port number respectively.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x28, 0x00]),
    /// #   hal::spi::Transaction::transfer(vec![0, 0, 0, 0], vec![0, 0, 0, 0]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, net::Ipv4Addr, Registers};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// let uipr = w5500.uipr()?;
    /// assert_eq!(uipr, Ipv4Addr::UNSPECIFIED);
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn uipr(&mut self) -> Result<Ipv4Addr, Self::Error> {
        let mut uipr = Ipv4Addr::UNSPECIFIED;
        self.read(reg::UIPR, COMMON_BLOCK_OFFSET, &mut uipr.octets)?;
        Ok(uipr)
    }

    /// Get the unreachable port.
    ///
    /// See [`Registers::uipr`] for more information.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x2C, 0x00]),
    /// #   hal::spi::Transaction::transfer(vec![0, 0], vec![0, 0]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, Registers};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// let uportr = w5500.uportr()?;
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn uportr(&mut self) -> Result<u16, Self::Error> {
        let mut buf: [u8; 2] = [0; 2];
        self.read(reg::UPORTR, COMMON_BLOCK_OFFSET, &mut buf)?;
        Ok(u16::from_be_bytes(buf))
    }

    /// Get the PHY configuration.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x2E, 0x00]),
    /// #   hal::spi::Transaction::transfer(vec![0], vec![0b10111000]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, PhyCfg, Registers};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// let phy_cfg: PhyCfg = w5500.phycfgr()?;
    /// assert_eq!(phy_cfg, PhyCfg::default());
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn phycfgr(&mut self) -> Result<PhyCfg, Self::Error> {
        let mut reg: [u8; 1] = [0];
        self.read(reg::PHYCFGR, COMMON_BLOCK_OFFSET, &mut reg)?;
        Ok(PhyCfg::from(reg[0]))
    }

    /// Set the PHY configuration.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x2E, 0x04]),
    /// #   hal::spi::Transaction::write(vec![0b11111000]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, OperationMode, PhyCfg, Registers};
    ///
    /// let mut phy_cfg: PhyCfg = PhyCfg::default();
    /// phy_cfg.set_opmdc(OperationMode::Auto);
    /// let mut w5500 = W5500::new(spi, pin);
    /// w5500.set_phycfgr(phy_cfg)?;
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn set_phycfgr(&mut self, phycfg: PhyCfg) -> Result<(), Self::Error> {
        self.write(reg::PHYCFGR, COMMON_BLOCK_OFFSET, &[phycfg.into()])
    }

    /// Get the version.
    ///
    /// The value returned is always `0x04`.
    ///
    /// This register is extremely useful as a sanity check to ensure SPI
    /// communications are working with the W5500.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x39, 0x00]),
    /// #   hal::spi::Transaction::transfer(vec![0], vec![0x04]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, Registers};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// let version = w5500.version()?;
    /// assert_eq!(version, 0x04);
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn version(&mut self) -> Result<u8, Self::Error> {
        let mut reg: [u8; 1] = [0];
        self.read(reg::VERSIONR, COMMON_BLOCK_OFFSET, &mut reg)?;
        Ok(reg[0])
    }

    /// Get the socket mode.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x00, 0x08]),
    /// #   hal::spi::Transaction::transfer(vec![0], vec![0x00]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, Registers, Socket, SocketMode};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// let socket_mode = w5500.sn_mr(Socket::Socket0)?;
    /// assert_eq!(socket_mode, SocketMode::default());
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn sn_mr(&mut self, socket: Socket) -> Result<SocketMode, Self::Error> {
        let mut reg: [u8; 1] = [0];
        self.read(reg::SN_MR, socket.block(), &mut reg)?;
        Ok(SocketMode::from(reg[0]))
    }

    /// Set the socket mode.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x00, 0x08 | 0x04]),
    /// #   hal::spi::Transaction::write(vec![0x01]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, Protocol, Registers, Socket, SocketMode};
    ///
    /// let mut socket_mode = SocketMode::default();
    /// socket_mode.set_protocol(Protocol::Tcp);
    /// let mut w5500 = W5500::new(spi, pin);
    /// w5500.set_sn_mr(Socket::Socket0, socket_mode)?;
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn set_sn_mr(&mut self, socket: Socket, mode: SocketMode) -> Result<(), Self::Error> {
        self.write(reg::SN_MR, socket.block(), &[mode.into()])
    }

    /// Get the socket command.
    ///
    /// The only use for reading this register is to check if a socket command
    /// has been accepted.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x01, 0x08 | 0x04]),
    /// #   hal::spi::Transaction::write(vec![0x01]),
    /// #   hal::spi::Transaction::write(vec![0x00, 0x01, 0x08]),
    /// #   hal::spi::Transaction::transfer(vec![0], vec![1]),
    /// #   hal::spi::Transaction::write(vec![0x00, 0x01, 0x08]),
    /// #   hal::spi::Transaction::transfer(vec![0], vec![0]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, Registers, Socket, SocketCommand};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// w5500.set_sn_cr(Socket::Socket0, SocketCommand::Open)?;
    /// loop {
    ///     if w5500.sn_cr(Socket::Socket0)? == SocketCommand::Accepted.into() {
    ///         break;
    ///     }
    /// }
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn sn_cr(&mut self, socket: Socket) -> Result<u8, Self::Error> {
        let mut reg: [u8; 1] = [0];
        self.read(reg::SN_CR, socket.block(), &mut reg)?;
        Ok(reg[0])
    }

    /// Set the socket command.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x01, 0x08 | 0x04]),
    /// #   hal::spi::Transaction::write(vec![0x01]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, Registers, Socket, SocketCommand};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// w5500.set_sn_cr(Socket::Socket0, SocketCommand::Open)?;
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn set_sn_cr(&mut self, socket: Socket, cmd: SocketCommand) -> Result<(), Self::Error> {
        self.write(reg::SN_CR, socket.block(), &[cmd.into()])
    }

    /// Get the socket interrupt status.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x02, 0x08]),
    /// #   hal::spi::Transaction::transfer(vec![0], vec![0]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, Registers, Socket};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// let socket_interrupts = w5500.sn_ir(Socket::Socket0)?;
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn sn_ir(&mut self, socket: Socket) -> Result<SocketInterrupt, Self::Error> {
        let mut reg: [u8; 1] = [0];
        self.read(reg::SN_IR, socket.block(), &mut reg)?;
        Ok(SocketInterrupt::from(reg[0]))
    }

    /// Set the socket interrupt status.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x02, 0x08]),
    /// #   hal::spi::Transaction::transfer(vec![0], vec![0]),
    /// #   hal::spi::Transaction::write(vec![0x00, 0x02, 0x08 | 0x04]),
    /// #   hal::spi::Transaction::write(vec![0]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, Registers, Socket};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// let socket_interrupts = w5500.sn_ir(Socket::Socket0)?;
    /// w5500.set_sn_ir(Socket::Socket0, socket_interrupts)?;
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn set_sn_ir(&mut self, socket: Socket, sn_ir: SocketInterrupt) -> Result<(), Self::Error> {
        self.write(reg::SN_IR, socket.block(), &[sn_ir.into()])
    }

    /// Get the socket status.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x03, 0x08]),
    /// #   hal::spi::Transaction::transfer(vec![0], vec![0]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use core::convert::TryFrom;
    /// use w5500_ll::{blocking::W5500, Registers, Socket, SocketStatus};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// let sn_sr_raw: u8 = w5500.sn_sr(Socket::Socket0)?;
    /// let sn_sr: SocketStatus = SocketStatus::try_from(sn_sr_raw).unwrap_or_default();
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn sn_sr(&mut self, socket: Socket) -> Result<u8, Self::Error> {
        let mut reg: [u8; 1] = [0];
        self.read(reg::SN_SR, socket.block(), &mut reg)?;
        Ok(reg[0])
    }

    /// Get the socket source port.
    ///
    /// This is only valid in TCP/UDP mode.
    /// This should be set before sending the OPEN command.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x04, 0x08]),
    /// #   hal::spi::Transaction::transfer(vec![0, 0], vec![0, 0]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, Registers, Socket, SocketMode};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// let socket_port: u16 = w5500.sn_port(Socket::Socket0)?;
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn sn_port(&mut self, socket: Socket) -> Result<u16, Self::Error> {
        let mut reg: [u8; 2] = [0; 2];
        self.read(reg::SN_PORT, socket.block(), &mut reg)?;
        Ok(u16::from_be_bytes(reg))
    }

    /// Set the socket source port.
    ///
    /// See [`Registers::sn_port`] for more information.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x04, 0x08 | 0x04]),
    /// #   hal::spi::Transaction::write(vec![0x00, 68]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, Registers, Socket};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// w5500.set_sn_port(Socket::Socket0, 68)?;
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn set_sn_port(&mut self, socket: Socket, port: u16) -> Result<(), Self::Error> {
        self.write(reg::SN_PORT, socket.block(), &u16::to_be_bytes(port))
    }

    /// Get the socket destination hardware address.
    ///
    /// Sn_DHAR configures the destination hardware address of Socket n when
    /// using SEND_MAC command in UDP mode or it indicates that it is acquired
    /// in ARP-process by CONNECT/SEND command.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x06, 0x08]),
    /// #   hal::spi::Transaction::transfer(
    /// #       vec![0, 0, 0, 0, 0, 0],
    /// #       vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]
    /// #   ),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, Registers, Socket};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// let dhar = w5500.sn_dhar(Socket::Socket0)?;
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn sn_dhar(&mut self, socket: Socket) -> Result<Eui48Addr, Self::Error> {
        let mut dhar: Eui48Addr = Eui48Addr::UNSPECIFIED;
        self.read(reg::SN_DHAR, socket.block(), &mut dhar.octets)?;
        Ok(dhar)
    }

    /// Set the socket destination hardware address.
    ///
    /// See [`Registers::sn_dhar`] for more information.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x06, 0x08 | 0x04]),
    /// #   hal::spi::Transaction::write(
    /// #       vec![0x12, 0x34, 0x00, 0x00, 0x00, 0x00]
    /// #   ),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, net::Eui48Addr, Registers, Socket};
    ///
    /// let dhar = Eui48Addr::new(0x12, 0x34, 0x00, 0x00, 0x00, 0x00);
    /// let mut w5500 = W5500::new(spi, pin);
    /// w5500.set_sn_dhar(Socket::Socket0, &dhar)?;
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn set_sn_dhar(&mut self, socket: Socket, dhar: &Eui48Addr) -> Result<(), Self::Error> {
        self.write(reg::SN_DHAR, socket.block(), &dhar.octets)
    }

    /// Get the socket destination IP address.
    ///
    /// This register configures or indicates the destination IP address.
    /// It it valid when the socket is in TCP/UDP mode.
    ///
    /// In TCP client mode it configures the TCP server address before the
    /// [`SocketCommand::Connect`] command.
    ///
    /// In TCP server mode it indicates the IP address of the TCP client after
    /// successfully establishing a connection.
    ///
    /// In UDP mode it configures an IP address of the peer to receive the UDP
    /// packet send by the [`SocketCommand::Send`] or [`SocketCommand::SendMac`]
    /// command.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x0C, 0x08]),
    /// #   hal::spi::Transaction::transfer(vec![0, 0, 0, 0], vec![0, 0, 0, 0]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, net::Ipv4Addr, Registers, Socket};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// let dipr = w5500.sn_dipr(Socket::Socket0)?;
    /// assert_eq!(dipr, Ipv4Addr::UNSPECIFIED);
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn sn_dipr(&mut self, socket: Socket) -> Result<Ipv4Addr, Self::Error> {
        let mut dipr: Ipv4Addr = Ipv4Addr::UNSPECIFIED;
        self.read(reg::SN_DIPR, socket.block(), &mut dipr.octets)?;
        Ok(dipr)
    }

    /// Set the socket destination IP address.
    ///
    /// See [`Registers::sn_dipr`] for more information.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x0C, 0x08 | 0x04]),
    /// #   hal::spi::Transaction::write(vec![192, 168, 0, 11]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, net::Ipv4Addr, Registers, Socket};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// w5500.set_sn_dipr(Socket::Socket0, &Ipv4Addr::new(192, 168, 0, 11))?;
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn set_sn_dipr(&mut self, socket: Socket, dipr: &Ipv4Addr) -> Result<(), Self::Error> {
        self.write(reg::SN_DIPR, socket.block(), &dipr.octets)
    }

    /// Get the socket destination port.
    ///
    /// This register configures or indicates the destination port number of
    /// the socket.
    /// It is valid when the socket is used in TCP/UDP mode.
    ///
    /// In TCP client mode, it configures the listen port number of the TCP
    /// server before the [`SocketCommand::Send`] command.
    ///
    /// In TCP server mode, it indicates the port number of the TCP client
    /// after successfully establishing connection.
    ///
    /// In UDP mode, it configures the port number of the peer to be transmitted
    /// in the UDP packet by the [`SocketCommand::Send`] or
    /// [`SocketCommand::SendMac`] command.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x10, 0x08]),
    /// #   hal::spi::Transaction::transfer(vec![0, 0], vec![0, 0]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, Registers, Socket, SocketMode};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// let socket_destination_port: u16 = w5500.sn_dport(Socket::Socket0)?;
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn sn_dport(&mut self, socket: Socket) -> Result<u16, Self::Error> {
        let mut reg: [u8; 2] = [0; 2];
        self.read(reg::SN_DPORT, socket.block(), &mut reg)?;
        Ok(u16::from_be_bytes(reg))
    }

    /// Set the socket destination port.
    ///
    /// See [`Registers::sn_dport`] for more information.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x10, 0x08 | 0x04]),
    /// #   hal::spi::Transaction::write(vec![0x00, 67]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, Registers, Socket};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// w5500.set_sn_dport(Socket::Socket0, 67)?;
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn set_sn_dport(&mut self, socket: Socket, port: u16) -> Result<(), Self::Error> {
        self.write(reg::SN_DPORT, socket.block(), &u16::to_be_bytes(port))
    }

    /// Get the socket maximum segment size.
    ///
    /// This register configures or indicates the MTU (Maximum Transfer Unit)
    /// of the socket.
    ///
    /// The default MTU is valid when the socket is used in TCP / UDP mode.
    /// However, when used in PPPoE mode it is determined within the PPPoE MTU.
    ///
    /// | Mode   | Normal Default | Normal Range | PPPoE Default | PPPoE Range |
    /// |--------|----------------|--------------|---------------|-------------|
    /// | TCP    | 1460           | 1 - 1460     | 1452          | 1 - 1452    |
    /// | UDP    | 1472           | 1 - 1472     | 1464          | 1 - 1464    |
    /// | MACRAW | 1514           | 1514         | 1514          | 1514        |
    ///
    /// When socket n is used in MACRAW mode, the default MTU is applied
    /// because the MTU is not processed internally.
    /// Therefore, when transmitting the data bigger than default MTU, the host
    /// should manually divide the data into the unit of default MTU.
    /// When socket n is used in TCP/UDP mode, and transmitting data bigger than
    /// the MTU, the data is automatically divided into the unit of MTU.
    ///
    /// In UDP mode, the configured MTU is used.
    /// When transmitting data to a peer with the different MTU size,
    /// the ICMP (Fragment MTU) packet might be received.
    /// In this case, IR(FMTU) becomes `1` and the peer information such as the
    /// MTU size and IP address is indicated from FMTUR and UIPR respectively.
    /// If IR\[MTU\] = `1`, the user cannot transmit data to the peer.
    ///
    /// To resume the communication with peer, do as followed.
    /// 1. Close socket n with the [`SocketCommand::Close`] command.
    /// 2. Set Sn_MSS to the indicated MTU from FMTUR
    /// 3. Open socket n with the [`SocketCommand::Open`] command.
    /// 4. Resume the communication with the peer.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x12, 0x08]),
    /// #   hal::spi::Transaction::transfer(vec![0, 0], vec![0x00, 0x00]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, Registers, Socket};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// let sn__mssr: u16 = w5500.sn_mssr(Socket::Socket0)?;
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn sn_mssr(&mut self, socket: Socket) -> Result<u16, Self::Error> {
        let mut reg: [u8; 2] = [0; 2];
        self.read(reg::SN_MSSR, socket.block(), &mut reg)?;
        Ok(u16::from_be_bytes(reg))
    }

    /// Set the socket maximum segment size.
    ///
    /// See [`Registers::sn_mssr`] for lots more information.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x12, 0x08 | 0x04]),
    /// #   hal::spi::Transaction::write(vec![0x05, 0xB4]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, Registers, Socket};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// w5500.set_sn_mssr(Socket::Socket0, 1460)?;
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn set_sn_mssr(&mut self, socket: Socket, mssr: u16) -> Result<(), Self::Error> {
        self.write(reg::SN_MSSR, socket.block(), &u16::to_be_bytes(mssr))
    }

    /// Get the IP type of service.
    ///
    /// This register configures the TOS (Type of service field in IP header)
    /// for socket n.
    /// Configure this field before sending the [`SocketCommand::Open`] command.
    ///
    /// For more details see [iana.org/assignments/ip-parameters].
    ///
    /// [iana.org/assignments/ip-parameters]: https://www.iana.org/assignments/ip-parameters
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x15, 0x08]),
    /// #   hal::spi::Transaction::transfer(vec![0], vec![0]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, Registers, Socket};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// let tos: u8 = w5500.sn_tos(Socket::Socket0)?;
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn sn_tos(&mut self, socket: Socket) -> Result<u8, Self::Error> {
        let mut reg: [u8; 1] = [0];
        self.read(reg::SN_TOS, socket.block(), &mut reg)?;
        Ok(reg[0])
    }

    /// Set the IP type of service.
    ///
    /// For more information see [`Registers::sn_tos`].
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x15, 0x08 | 0x04]),
    /// #   hal::spi::Transaction::write(vec![0x01]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, Registers, Socket};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// w5500.set_sn_tos(Socket::Socket0, 1)?;
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn set_sn_tos(&mut self, socket: Socket, tos: u8) -> Result<(), Self::Error> {
        self.write(reg::SN_TOS, socket.block(), &[tos])
    }

    /// Get the time to live.
    ///
    /// This register configures the TTL (Time to Live field in the IP header)
    /// for socket n.
    ///
    /// For more details see [iana.org/assignments/ip-parameters].
    ///
    /// [iana.org/assignments/ip-parameters]: https://www.iana.org/assignments/ip-parameters
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x16, 0x08]),
    /// #   hal::spi::Transaction::transfer(vec![0], vec![0x80]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, Registers, Socket};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// let ttl: u8 = w5500.sn_ttl(Socket::Socket0)?;
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn sn_ttl(&mut self, socket: Socket) -> Result<u8, Self::Error> {
        let mut reg: [u8; 1] = [0];
        self.read(reg::SN_TTL, socket.block(), &mut reg)?;
        Ok(reg[0])
    }

    /// Set the time to live.
    ///
    /// For more information see [`Registers::sn_ttl`].
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x16, 0x08 | 0x04]),
    /// #   hal::spi::Transaction::write(vec![0x80]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, Registers, Socket};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// w5500.set_sn_ttl(Socket::Socket0, 0x80)?;
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn set_sn_ttl(&mut self, socket: Socket, ttl: u8) -> Result<(), Self::Error> {
        self.write(reg::SN_TTL, socket.block(), &[ttl])
    }

    /// Get the socket RX buffer size.
    ///
    /// The buffer size can be configured to any of the sizes in [`BufferSize`].
    ///
    /// The sum of all the socket RX buffers cannot exceed 16 KiB.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x1E, 0x08]),
    /// #   hal::spi::Transaction::transfer(vec![0], vec![0x02]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use core::convert::TryFrom;
    /// use w5500_ll::{blocking::W5500, BufferSize, Registers, Socket};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// let sn_rxbuf_size_raw: u8 = w5500.sn_rxbuf_size(Socket::Socket0)?;
    /// assert_eq!(BufferSize::try_from(sn_rxbuf_size_raw), Ok(BufferSize::KB2));
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn sn_rxbuf_size(&mut self, socket: Socket) -> Result<u8, Self::Error> {
        let mut reg: [u8; 1] = [0];
        self.read(reg::SN_RXBUF_SIZE, socket.block(), &mut reg)?;
        Ok(reg[0])
    }

    /// Set the socket RX buffer size.
    ///
    /// See [`Registers::sn_rxbuf_size`] for more information.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x1E, 0x08 | 0x04]),
    /// #   hal::spi::Transaction::write(vec![1]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use core::convert::TryFrom;
    /// use w5500_ll::{blocking::W5500, BufferSize, Registers, Socket};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// w5500.set_sn_rxbuf_size(Socket::Socket0, BufferSize::KB1)?;
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn set_sn_rxbuf_size(&mut self, socket: Socket, size: BufferSize) -> Result<(), Self::Error> {
        self.write(reg::SN_RXBUF_SIZE, socket.block(), &[size.into()])
    }

    /// Get the socket TX buffer size.
    ///
    /// The buffer size can be configured to any of the sizes in [`BufferSize`].
    ///
    /// The sum of all the socket TX buffers cannot exceed 16 KiB.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x1F, 0x08]),
    /// #   hal::spi::Transaction::transfer(vec![0], vec![0x02]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use core::convert::TryFrom;
    /// use w5500_ll::{blocking::W5500, BufferSize, Registers, Socket};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// let sn_txbuf_size_raw: u8 = w5500.sn_txbuf_size(Socket::Socket0)?;
    /// assert_eq!(BufferSize::try_from(sn_txbuf_size_raw), Ok(BufferSize::KB2));
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn sn_txbuf_size(&mut self, socket: Socket) -> Result<u8, Self::Error> {
        let mut reg: [u8; 1] = [0];
        self.read(reg::SN_TXBUF_SIZE, socket.block(), &mut reg)?;
        Ok(reg[0])
    }

    /// Set the socket TX buffer size.
    ///
    /// See [`Registers::sn_rxbuf_size`] for more information.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x1F, 0x08 | 0x04]),
    /// #   hal::spi::Transaction::write(vec![1]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use core::convert::TryFrom;
    /// use w5500_ll::{blocking::W5500, BufferSize, Registers, Socket};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// w5500.set_sn_txbuf_size(Socket::Socket0, BufferSize::KB1)?;
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn set_sn_txbuf_size(&mut self, socket: Socket, size: BufferSize) -> Result<(), Self::Error> {
        self.write(reg::SN_TXBUF_SIZE, socket.block(), &[size.into()])
    }

    /// Get transmit buffer free size.
    ///
    /// This register indicates the free size of socket n TX buffer.
    /// It is initialized to the configured size by [`Registers::sn_txbuf_size`].
    /// Data bigger than Sn_TX_FSR should not be written to the TX buffer to
    /// prevent buffer overflow.
    ///
    /// Check this register before writing data to the socket TX buffer,
    /// and if data is equal or smaller than its checked size, transmit the data
    /// with the [`SocketCommand::Send`] or [`SocketCommand::SendMac`] command
    /// after saving the data in Socket n TX buffer.
    ///
    /// If data is bigger than its checked size, transmit the data after
    /// dividing into the checked size and saving in the socket TX buffer.
    ///
    /// If [`Registers::sn_mr`] is not in TCP mode, this register is
    /// automatically calculated as the difference between
    /// [`Registers::sn_tx_wr`] and [`Registers::sn_tx_rd`].
    ///
    /// If [`Registers::sn_mr`] is in TCP mode, this register is automatically
    /// calculated as the difference between the internal ACK pointer which
    /// indicates the point of data is received already by the connected
    /// peer.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x20, 0x08]),
    /// #   hal::spi::Transaction::transfer(vec![0, 0], vec![0x08, 0x00]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, Registers, Socket, SocketMode};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// let sn_tx_fsr: u16 = w5500.sn_tx_fsr(Socket::Socket0)?;
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn sn_tx_fsr(&mut self, socket: Socket) -> Result<u16, Self::Error> {
        let mut reg: [u8; 2] = [0; 2];
        self.read(reg::SN_TX_FSR, socket.block(), &mut reg)?;
        Ok(u16::from_be_bytes(reg))
    }

    /// Get the socket TX read pointer.
    ///
    /// This register is initialized by the [`SocketCommand::Open`] command.
    /// However, in TCP mode, this is re-initialized while connecting with TCP.
    ///
    /// After initialization, this is auto-increased by the
    /// [`SocketCommand::Send`] command.
    ///
    /// The [`SocketCommand::Send`] command transmits the saved data from the
    /// current [`Registers::sn_tx_rd`] to the [`Registers::sn_tx_wr`] in the
    /// socket n TX buffer.
    /// After transmitting the saved data, the [`SocketCommand::Send`] command
    /// increases [`Registers::sn_tx_rd`] the as same as
    /// [`Registers::sn_tx_wr`].
    ///
    /// If its increment value exceeds the maximum value 0xFFFF,
    /// (greater than 0x10000 and the carry bit occurs), then the carry bit is
    /// ignored and will automatically update with the lower 16bits value.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x22, 0x08]),
    /// #   hal::spi::Transaction::transfer(vec![0, 0], vec![0x00, 0x00]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, Registers, Socket, SocketMode};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// let sn_tx_rd: u16 = w5500.sn_tx_rd(Socket::Socket0)?;
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn sn_tx_rd(&mut self, socket: Socket) -> Result<u16, Self::Error> {
        let mut reg: [u8; 2] = [0; 2];
        self.read(reg::SN_TX_RD, socket.block(), &mut reg)?;
        Ok(u16::from_be_bytes(reg))
    }

    /// Get the socket TX write pointer.
    ///
    /// This register is initialized by the [`SocketCommand::Open`] command.
    /// However, in TCP mode, this is re-initialized while connecting with TCP.
    ///
    /// This should be updated as follows:
    /// 1. Read the starting address for transmitting data.
    /// 2. Write data to the socket TX buffer buffer.
    /// 3. Update this register by the number of bytes written to the TX buffer.
    ///    Allow wrapping to occur upon `u16` overflow.
    /// 4. Transmit the saved data in the socket TX buffer by using the
    ///    [`SocketCommand::Send`] command.
    ///
    /// See [`Registers::set_sn_tx_buf`] for an additional example.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x24, 0x08]),
    /// #   hal::spi::Transaction::transfer(vec![0, 0], vec![0x00, 0x00]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, Registers, Socket};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// let sn_tx_wr: u16 = w5500.sn_tx_wr(Socket::Socket0)?;
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn sn_tx_wr(&mut self, socket: Socket) -> Result<u16, Self::Error> {
        let mut reg: [u8; 2] = [0; 2];
        self.read(reg::SN_TX_WR, socket.block(), &mut reg)?;
        Ok(u16::from_be_bytes(reg))
    }

    /// Set the socket TX write pointer.
    ///
    /// See [`Registers::sn_tx_wr`] for more information.
    ///
    /// See [`Registers::set_sn_tx_buf`] for an example.
    fn set_sn_tx_wr(&mut self, socket: Socket, ptr: u16) -> Result<(), Self::Error> {
        self.write(reg::SN_TX_WR, socket.block(), &ptr.to_be_bytes())
    }

    /// Get the socket received data size.
    ///
    /// This register indicates the data size received and saved in the socket
    /// RX buffer.
    /// This register does not exceed the configured size
    /// ([`Registers::set_sn_rxbuf_size`]) and is calculated as the difference
    /// between [`Registers::sn_rx_wr`] and [`Registers::sn_rx_rd`].
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x26, 0x08]),
    /// #   hal::spi::Transaction::transfer(vec![0, 0], vec![0x00, 0x00]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, Registers, Socket};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// let sn_rx_rsr: u16 = w5500.sn_rx_rsr(Socket::Socket0)?;
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn sn_rx_rsr(&mut self, socket: Socket) -> Result<u16, Self::Error> {
        let mut reg: [u8; 2] = [0; 2];
        self.read(reg::SN_RX_RSR, socket.block(), &mut reg)?;
        Ok(u16::from_be_bytes(reg))
    }

    /// Get the socket read data pointer.
    ///
    /// This register is initialized by the [`SocketCommand::Open`] command.
    ///
    /// This should be updated as follows:
    /// 1. Read the starting address for reading data.
    /// 2. Read from the socket RX buffer.
    /// 3. Update this register by the number of bytes read from the RX buffer.
    ///    Allow wrapping to occur upon `u16` overflow.
    /// 4. Send a [`SocketCommand::Recv`] command to notify the W5500 of the
    ///    retrieved data.
    ///
    /// See [`Registers::sn_rx_buf`] for an additional example.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x28, 0x08]),
    /// #   hal::spi::Transaction::transfer(vec![0, 0], vec![0x00, 0x00]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, Registers, Socket};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// let sn_rx_rd: u16 = w5500.sn_rx_rd(Socket::Socket0)?;
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn sn_rx_rd(&mut self, socket: Socket) -> Result<u16, Self::Error> {
        let mut reg: [u8; 2] = [0; 2];
        self.read(reg::SN_RX_RD, socket.block(), &mut reg)?;
        Ok(u16::from_be_bytes(reg))
    }

    /// Set the socket read data pointer.
    ///
    /// See [`Registers::sn_rx_rd`] for more information.
    /// See [`Registers::sn_rx_buf`] for an example.
    fn set_sn_rx_rd(&mut self, socket: Socket, ptr: u16) -> Result<(), Self::Error> {
        self.write(reg::SN_RX_RD, socket.block(), &ptr.to_be_bytes())
    }

    /// Get the socket RX write pointer.
    ///
    /// This register is initialized by the [`SocketCommand::Open`] command, and
    /// it is auto-incremented by hardware.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x2A, 0x08]),
    /// #   hal::spi::Transaction::transfer(vec![0, 0], vec![0x00, 0x00]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, Registers, Socket};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// let sn_rx_wr: u16 = w5500.sn_rx_wr(Socket::Socket0)?;
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn sn_rx_wr(&mut self, socket: Socket) -> Result<u16, Self::Error> {
        let mut reg: [u8; 2] = [0; 2];
        self.read(reg::SN_RX_WR, socket.block(), &mut reg)?;
        Ok(u16::from_be_bytes(reg))
    }

    /// Get the socket interrupt mask.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x2C, 0x08]),
    /// #   hal::spi::Transaction::transfer(vec![0], vec![0xFF]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, Registers, Socket, SocketInterruptMask};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// let sn_imr: SocketInterruptMask = w5500.sn_imr(Socket::Socket0)?;
    /// assert_eq!(sn_imr, SocketInterruptMask::default());
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn sn_imr(&mut self, socket: Socket) -> Result<SocketInterruptMask, Self::Error> {
        let mut reg: [u8; 1] = [0];
        self.read(reg::SN_IMR, socket.block(), &mut reg)?;
        Ok(SocketInterruptMask::from(reg[0]))
    }

    /// Set the socket interrupt mask.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x2C, 0x08 | 0x04]),
    /// #   hal::spi::Transaction::write(vec![0xE0]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, Registers, Socket, SocketInterruptMask};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// w5500.set_sn_imr(Socket::Socket0, SocketInterruptMask::ALL_MASKED)?;
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn set_sn_imr(&mut self, socket: Socket, mask: SocketInterruptMask) -> Result<(), Self::Error> {
        self.write(reg::SN_IMR, socket.block(), &[mask.into()])
    }

    /// Get the socket fragment.
    ///
    /// This configures the fragment field in the IP header.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x2D, 0x08]),
    /// #   hal::spi::Transaction::transfer(vec![0, 0], vec![0x40, 0x00]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, Registers, Socket};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// let frag: u16 = w5500.sn_frag(Socket::Socket0)?;
    /// assert_eq!(frag, 0x4000);
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn sn_frag(&mut self, socket: Socket) -> Result<u16, Self::Error> {
        let mut buf: [u8; 2] = [0; 2];
        self.read(reg::SN_FRAG, socket.block(), &mut buf)?;
        Ok(u16::from_be_bytes(buf))
    }

    /// Set the socket fragment.
    ///
    /// See [`Registers::sn_frag`] for more information.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x2D, 0x08 | 0x04]),
    /// #   hal::spi::Transaction::write(vec![0x00, 0x00]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, Registers, Socket};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// w5500.set_sn_frag(Socket::Socket0, 0)?;
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn set_sn_frag(&mut self, socket: Socket, frag: u16) -> Result<(), Self::Error> {
        self.write(reg::SN_FRAG, socket.block(), &u16::to_be_bytes(frag))
    }

    /// Get the socket keep alive time.
    ///
    /// This register configures the transmitting timer of the keep alive (KA)
    /// packet for the socket.  This is valid only in TCP mode, and is ignored
    /// in all other modes.
    ///
    /// The time unit is 5s.
    ///
    /// The KA packet is transmittable after [`Registers::sn_sr`] is changed to
    /// [`SocketStatus::Established`] and after the data is transmitted or
    /// received to/from a peer at least once.
    ///
    /// In the case of a non-zero keep alive value the W5500 automatically
    /// transmits a KA packet after time-period for checking the TCP connection
    /// (automatic-keepalive-process).
    ///
    /// In case of a zero keep alive value, the keep alive packet can be
    /// transmitted with [`SocketCommand::SendKeep`].  This command is ignored
    /// for non-zero keep alive values.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x2F, 0x08]),
    /// #   hal::spi::Transaction::transfer(vec![0], vec![0]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, Registers, Socket};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// let sn_kpalvtr: u8 = w5500.sn_kpalvtr(Socket::Socket0)?;
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn sn_kpalvtr(&mut self, socket: Socket) -> Result<u8, Self::Error> {
        let mut buf: [u8; 1] = [0];
        self.read(reg::SN_KPALVTR, socket.block(), &mut buf)?;
        Ok(buf[0])
    }

    /// Set the socket keep alive time.
    ///
    /// See [`Registers::sn_kpalvtr`] for more information.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, 0x2F, 0x08 | 0x04]),
    /// #   hal::spi::Transaction::write(vec![0x0A]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use w5500_ll::{blocking::W5500, Registers, Socket};
    ///
    /// let mut w5500 = W5500::new(spi, pin);
    /// // 50s keep alive timer
    /// w5500.set_sn_kpalvtr(Socket::Socket0, 10)?;
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn set_sn_kpalvtr(&mut self, socket: Socket, kpalvtr: u8) -> Result<(), Self::Error> {
        self.write(reg::SN_KPALVTR, socket.block(), &[kpalvtr])
    }

    /// Write the socket TX buffer.
    ///
    /// # Example
    ///
    /// ```
    /// use core::convert::TryFrom;
    /// use w5500_ll::{blocking::W5500, Registers, Socket, SocketCommand};
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, w5500_ll::reg::SN_TX_FSR as u8, 0x08]),
    /// #   hal::spi::Transaction::transfer(vec![0, 0], vec![0x08, 0x00]),
    /// #   hal::spi::Transaction::write(vec![0x00, w5500_ll::reg::SN_TX_WR as u8, 0x08]),
    /// #   hal::spi::Transaction::transfer(vec![0, 0], vec![0x00, 0x00]),
    /// #   hal::spi::Transaction::write(vec![0x00, 0x00, (Socket::Socket0.tx_block() as u8) << 3 | 0x04]),
    /// #   hal::spi::Transaction::write(vec![0x12, 0x34, 0x56, 0x78, 0x9A]),
    /// #   hal::spi::Transaction::write(vec![0x00, 0x24, 0x08 | 0x04]),
    /// #   hal::spi::Transaction::write(vec![0x00, 5]),
    /// #   hal::spi::Transaction::write(vec![0x00, 0x01, 0x08 | 0x04]),
    /// #   hal::spi::Transaction::write(vec![SocketCommand::Send.into()]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// # let mut w5500 = W5500::new(spi, pin);
    ///
    /// // the socket should already be opened at this point
    /// const THE_SOCKET: Socket = Socket::Socket0;
    ///
    /// let data: [u8; 5] = [0x12, 0x34, 0x56, 0x78, 0x9A];
    /// // panics if the data length exceeds u16 capacity
    /// let data_len: u16 = u16::try_from(data.len()).expect("data length exceeds u16::MAX");
    ///
    /// let free_size: u16 = w5500.sn_tx_fsr(THE_SOCKET)?;
    /// // panics if the buffer size exceeds free size
    /// assert!(free_size >= data_len, "TX buffer overflow free size: {}, data size: {}", free_size, data_len);
    ///
    /// let mut ptr: u16 = w5500.sn_tx_wr(THE_SOCKET)?;
    /// w5500.set_sn_tx_buf(THE_SOCKET, ptr, &data)?;
    /// ptr += data_len; // wrapping on overflow is intentional
    /// w5500.set_sn_tx_wr(THE_SOCKET, ptr)?;
    /// w5500.set_sn_cr(THE_SOCKET, SocketCommand::Send)?;
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn set_sn_tx_buf(&mut self, socket: Socket, ptr: u16, buf: &[u8]) -> Result<(), Self::Error> {
        self.write(ptr, socket.tx_block(), buf)
    }

    /// Read the socket RX buffer.
    ///
    /// # Example
    ///
    /// ```
    /// use core::convert::TryFrom;
    /// use w5500_ll::{blocking::W5500, Registers, Socket, SocketCommand};
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![0x00, w5500_ll::reg::SN_RX_RSR as u8, 0x08]),
    /// #   hal::spi::Transaction::transfer(vec![0, 0], vec![0, 4]),
    /// #   hal::spi::Transaction::write(vec![0x00, w5500_ll::reg::SN_RX_RD as u8, 0x08]),
    /// #   hal::spi::Transaction::transfer(vec![0, 0], vec![0, 0]),
    /// #   hal::spi::Transaction::write(vec![0x00, 0x00, (Socket::Socket0.rx_block() as u8) << 3]),
    /// #   hal::spi::Transaction::transfer(vec![0, 0, 0, 0], vec![0, 0, 0, 0]),
    /// #   hal::spi::Transaction::write(vec![0x00, w5500_ll::reg::SN_RX_RD as u8, 0x08 | 0x04]),
    /// #   hal::spi::Transaction::write(vec![0, 4]),
    /// #   hal::spi::Transaction::write(vec![0x00, w5500_ll::reg::SN_CR as u8, 0x08 | 0x04]),
    /// #   hal::spi::Transaction::write(vec![SocketCommand::Recv.into()]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// # let mut w5500 = W5500::new(spi, pin);
    ///
    /// // the socket should already be opened at this point
    /// // a socket interrupt will indicate there is data to be retrieved
    /// const THE_SOCKET: Socket = Socket::Socket0;
    ///
    /// // in reality you will need a larger buffer for most protocols
    /// const BUF_LEN: usize = 8;
    /// let mut buf: [u8; BUF_LEN] = [0; BUF_LEN];
    ///
    /// let rsr: u16 = w5500.sn_rx_rsr(THE_SOCKET)?;
    /// // panics if we incorrectly though there was data to read
    /// debug_assert_ne!(rsr, 0);
    /// // panics if the recieve data is larger than the local buffer
    /// assert!(BUF_LEN >= usize::from(rsr), "RX buffer overflow buffer len: {}, rsr: {}", BUF_LEN, rsr);
    ///
    /// let mut ptr: u16 = w5500.sn_rx_rd(THE_SOCKET)?;
    /// w5500.sn_rx_buf(THE_SOCKET, ptr, &mut buf[..rsr.into()])?;
    /// ptr += rsr; // wrapping on overflow is intentional
    /// w5500.set_sn_rx_rd(THE_SOCKET, ptr)?;
    /// w5500.set_sn_cr(THE_SOCKET, SocketCommand::Recv)?;
    /// # Ok::<(), w5500_ll::blocking::Error<_, _>>(())
    /// ```
    fn sn_rx_buf(&mut self, socket: Socket, ptr: u16, buf: &mut [u8]) -> Result<(), Self::Error> {
        self.read(ptr, socket.rx_block(), buf)
    }
}
