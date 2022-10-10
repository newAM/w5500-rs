//! Asynchronous W5500 traits.
use crate::{
    BufferSize, Eui48Addr, Interrupt, Ipv4Addr, Mode, PhyCfg, Reg, Sn, SnReg, SocketAddrV4,
    SocketCommand, SocketInterrupt, SocketInterruptMask, SocketMode, SocketStatus,
    COMMON_BLOCK_OFFSET,
};
use core::future::Future;

type MrFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<Mode, S::Error>>;
type SetMrFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<(), S::Error>>;

type GarFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<Ipv4Addr, S::Error>>;
type SetGarFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<(), S::Error>>;

type SubrFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<Ipv4Addr, S::Error>>;
type SetSubrFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<(), S::Error>>;

type SharFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<Eui48Addr, S::Error>>;
type SetSharFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<(), S::Error>>;

type SiprFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<Ipv4Addr, S::Error>>;
type SetSiprFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<(), S::Error>>;

type IntlevelFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<u16, S::Error>>;
type SetIntlevelFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<(), S::Error>>;

type IrFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<Interrupt, S::Error>>;
type SetIrFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<(), S::Error>>;

type ImrFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<Interrupt, S::Error>>;
type SetImrFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<(), S::Error>>;

type SirFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<u8, S::Error>>;

type SimrFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<u8, S::Error>>;
type SetSimrFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<(), S::Error>>;

type RtrFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<u16, S::Error>>;
type SetRtrFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<(), S::Error>>;

type RcrFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<u8, S::Error>>;
type SetRcrFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<(), S::Error>>;

type PtimerFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<u8, S::Error>>;
type SetPtimerFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<(), S::Error>>;

type PmagicFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<u8, S::Error>>;
type SetPmagicFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<(), S::Error>>;

type PharFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<Eui48Addr, S::Error>>;
type SetPharFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<(), S::Error>>;

type PsidFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<u16, S::Error>>;
type SetPsidFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<(), S::Error>>;

type PmruFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<u16, S::Error>>;
type SetPmruFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<(), S::Error>>;

type UiprFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<Ipv4Addr, S::Error>>;

type UportrFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<u16, S::Error>>;

type PhycfgrFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<PhyCfg, S::Error>>;
type SetPhycfgrFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<(), S::Error>>;

type VersionFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<u8, S::Error>>;

type SnMrFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<SocketMode, S::Error>>;
type SetSnMrFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<(), S::Error>>;

type SnCrFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<u8, S::Error>>;
type SetSnCrFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<(), S::Error>>;

type SnIrFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<SocketInterrupt, S::Error>>;
type SetSnIrFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<(), S::Error>>;

type SnSrFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<Result<SocketStatus, u8>, S::Error>>;

type SnPortFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<u16, S::Error>>;
type SetSnPortFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<(), S::Error>>;

type SnDharFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<Eui48Addr, S::Error>>;
type SetSnDharFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<(), S::Error>>;

type SnDiprFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<Ipv4Addr, S::Error>>;
type SetSnDiprFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<(), S::Error>>;

type SnDportFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<u16, S::Error>>;
type SetSnDportFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<(), S::Error>>;

type SnDestFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<SocketAddrV4, S::Error>>;
type SetSnDestFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<(), S::Error>>;

type SnMssrFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<u16, S::Error>>;
type SetSnMssrFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<(), S::Error>>;

type SnTosFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<u8, S::Error>>;
type SetSnTosFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<(), S::Error>>;

type SnTtlFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<u8, S::Error>>;
type SetSnTtlFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<(), S::Error>>;

type SnRxbufSizeFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<Result<BufferSize, u8>, S::Error>>;
type SetSnRxbufSizeFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<(), S::Error>>;

type SnTxbufSizeFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<Result<BufferSize, u8>, S::Error>>;
type SetSnTxbufSizeFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<(), S::Error>>;

type SnTxFsrFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<u16, S::Error>>;

type SnTxRdFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<u16, S::Error>>;

type SnTxWrFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<u16, S::Error>>;
type SetSnTxWrFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<(), S::Error>>;

type SnRxRsrFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<u16, S::Error>>;

type SnRxRdFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<u16, S::Error>>;
type SetSnRxRdFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<(), S::Error>>;

type SnRxWrFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<u16, S::Error>>;

type SnImrFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<SocketInterruptMask, S::Error>>;
type SetSnImrFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<(), S::Error>>;

type SnFragFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<u16, S::Error>>;
type SetSnFragFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<(), S::Error>>;

type SnKpalvtrFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<u8, S::Error>>;
type SetSnKpalvtrFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<(), S::Error>>;

type SetSnTxBufFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<(), S::Error>>;

type SnTxBufFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<(), S::Error>>;

type SnRxBufFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<(), S::Error>>;

type SetSnRxBufFuture<'a, S>
where
    S: Registers + ?Sized + 'a,
= impl Future<Output = Result<(), S::Error>>;

/// Asynchronous register IO.
pub trait Registers {
    /// Register accessor error type.
    type Error;

    /// Future for [`Registers::read`].
    type ReadFuture<'a>: Future<Output = Result<(), Self::Error>>
    where
        Self: 'a;

    /// Read from the W5500 asynchronously.
    ///
    /// # Arguments
    ///
    /// * `addr` - Starting address of the memory being read.
    /// * `block` - W5500 block select bits
    /// * `data` - Buffer to read data into. The number of bytes read is equal
    ///   to the length of this buffer.
    fn read<'a>(&'a mut self, addr: u16, block: u8, data: &'a mut [u8]) -> Self::ReadFuture<'a>;

    /// Future for [`Registers::write`].
    type WriteFuture<'a>: Future<Output = Result<(), Self::Error>>
    where
        Self: 'a;

    /// Write to the W5500 asynchronously.
    ///
    /// # Arguments
    ///
    /// * `addr` - Starting address of the memory being written.
    /// * `block` - W5500 block select bits
    /// * `data` - Buffer of data to write. The number of bytes written is equal
    ///   to the length of this buffer.
    fn write<'a>(&'a mut self, addr: u16, block: u8, data: &'a [u8]) -> Self::WriteFuture<'a>;

    /// Get the mode register.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x00, 0x00]),
    /// #   ehm1::spi::Transaction::async_read(0),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500, Mode};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// let mode: Mode = w5500.mr().await?;
    /// assert_eq!(mode, Mode::default());
    /// # Ok(()) }
    /// ```
    fn mr(&mut self) -> MrFuture<'_, Self> {
        async move {
            let mut reg: [u8; 1] = [0];
            self.read(Reg::MR.addr(), COMMON_BLOCK_OFFSET, &mut reg)
                .await
                .map(|_| Mode::from(reg[0]))
        }
    }

    /// Set the mode register.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x00, 0x04]),
    /// #   ehm1::spi::Transaction::async_write(w5500_ll::Mode::WOL_MASK),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500, Mode};
    ///
    /// const MODE: Mode = Mode::DEFAULT.enable_wol();
    /// let mut w5500 = W5500::new(spi);
    /// w5500.set_mr(MODE).await?;
    /// # Ok(()) }
    /// ```
    fn set_mr(&mut self, mode: Mode) -> SetMrFuture<'_, Self> {
        async move {
            self.write(Reg::MR.addr(), COMMON_BLOCK_OFFSET, &[mode.into()])
                .await
        }
    }

    /// Get the gateway IP address.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x01, 0x00]),
    /// #   ehm1::spi::Transaction::async_read_vec(vec![0, 0, 0, 0]),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500, net::Ipv4Addr};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// let gar = w5500.gar().await?;
    /// assert_eq!(gar, Ipv4Addr::UNSPECIFIED);
    /// # Ok(()) }
    /// ```
    fn gar(&mut self) -> GarFuture<'_, Self> {
        async move {
            let mut gar = Ipv4Addr::UNSPECIFIED;
            self.read(Reg::GAR0.addr(), COMMON_BLOCK_OFFSET, &mut gar.octets)
                .await?;
            Ok(gar)
        }
    }

    /// Set the gateway IP address.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x01, 0x04]),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![192, 168, 0, 1]),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500, net::Ipv4Addr};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// w5500.set_gar(&Ipv4Addr::new(192, 168, 0, 1)).await?;
    /// # Ok(()) }
    /// ```
    fn set_gar<'a>(&'a mut self, gar: &'a Ipv4Addr) -> SetGarFuture<'a, Self> {
        async move {
            self.write(Reg::GAR0.addr(), COMMON_BLOCK_OFFSET, &gar.octets)
                .await
        }
    }

    /// Get the subnet mask.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x05, 0x00]),
    /// #   ehm1::spi::Transaction::async_read_vec(vec![0, 0, 0, 0]),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500, net::Ipv4Addr};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// let subr = w5500.subr().await?;
    /// assert_eq!(subr, Ipv4Addr::UNSPECIFIED);
    /// # Ok(()) }
    /// ```
    fn subr(&mut self) -> SubrFuture<'_, Self> {
        async move {
            let mut subr = Ipv4Addr::UNSPECIFIED;
            self.read(Reg::SUBR0.addr(), COMMON_BLOCK_OFFSET, &mut subr.octets)
                .await?;
            Ok(subr)
        }
    }

    /// Set the subnet mask.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x05, 0x04]),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![255, 255, 255, 0]),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500, net::Ipv4Addr};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// w5500.set_subr(&Ipv4Addr::new(255, 255, 255, 0)).await?;
    /// # Ok(()) }
    /// ```
    fn set_subr<'a>(&'a mut self, subr: &'a Ipv4Addr) -> SetSubrFuture<'a, Self> {
        async move {
            self.write(Reg::SUBR0.addr(), COMMON_BLOCK_OFFSET, &subr.octets)
                .await
        }
    }

    /// Get the source hardware address.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x09, 0x00]),
    /// #   ehm1::spi::Transaction::async_read_vec(vec![0, 0, 0, 0, 0, 0]),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500, net::Eui48Addr};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// let shar = w5500.shar().await?;
    /// assert_eq!(shar, Eui48Addr::UNSPECIFIED);
    /// # Ok(()) }
    /// ```
    fn shar(&mut self) -> SharFuture<'_, Self> {
        async move {
            let mut shar = Eui48Addr::UNSPECIFIED;
            self.read(Reg::SHAR0.addr(), COMMON_BLOCK_OFFSET, &mut shar.octets)
                .await?;
            Ok(shar)
        }
    }

    /// Set the source hardware address.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x09, 0x04]),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x12, 0x34, 0, 0, 0, 0]),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500, net::Eui48Addr};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// w5500
    ///     .set_shar(&Eui48Addr::new(0x12, 0x34, 0x00, 0x00, 0x00, 0x00))
    ///     .await?;
    /// # Ok(()) }
    /// ```
    fn set_shar<'a>(&'a mut self, shar: &'a Eui48Addr) -> SetSharFuture<'a, Self> {
        async {
            self.write(Reg::SHAR0.addr(), COMMON_BLOCK_OFFSET, &shar.octets)
                .await
        }
    }

    /// Get the source (client) IP address.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x0F, 0x00]),
    /// #   ehm1::spi::Transaction::async_read_vec(vec![0, 0, 0, 0]),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500, net::Ipv4Addr};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// let sipr = w5500.sipr().await?;
    /// assert_eq!(sipr, Ipv4Addr::UNSPECIFIED);
    /// # Ok(()) }
    /// ```
    fn sipr(&'_ mut self) -> SiprFuture<'_, Self> {
        async move {
            let mut sipr = Ipv4Addr::UNSPECIFIED;
            self.read(Reg::SIPR0.addr(), COMMON_BLOCK_OFFSET, &mut sipr.octets)
                .await?;
            Ok(sipr)
        }
    }

    /// Set the source (client) IP address.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x0F, 0x04]),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![192, 168, 0, 150]),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500, net::Ipv4Addr};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// w5500.set_sipr(&Ipv4Addr::new(192, 168, 0, 150)).await?;
    /// # Ok(()) }
    /// ```
    fn set_sipr<'a>(&'a mut self, sipr: &'a Ipv4Addr) -> SetSiprFuture<'a, Self> {
        async move {
            self.write(Reg::SIPR0.addr(), COMMON_BLOCK_OFFSET, &sipr.octets)
                .await
        }
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
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x13, 0x00]),
    /// #   ehm1::spi::Transaction::async_read_vec(vec![0, 0]),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// let intlevel: u16 = w5500.intlevel().await?;
    /// assert_eq!(intlevel, 0x00);
    /// # Ok(()) }
    /// ```
    fn intlevel(&mut self) -> IntlevelFuture<'_, Self> {
        async move {
            let mut buf: [u8; 2] = [0; 2];
            self.read(Reg::INTLEVEL0.addr(), COMMON_BLOCK_OFFSET, &mut buf)
                .await?;
            Ok(u16::from_be_bytes(buf))
        }
    }

    /// Set the interrupt low level time.
    ///
    /// See [`Registers::intlevel`] for more information.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x13, 0x04]),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x12, 0x34]),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// w5500.set_intlevel(0x1234).await?;
    /// # Ok(()) }
    /// ```
    fn set_intlevel(&mut self, intlevel: u16) -> SetIntlevelFuture<'_, Self> {
        async move {
            self.write(
                Reg::INTLEVEL0.addr(),
                COMMON_BLOCK_OFFSET,
                &intlevel.to_be_bytes(),
            )
            .await
        }
    }

    /// Get the interrupt status.
    ///
    /// `1` indicates the interrupt is raised.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x15, 0x00]),
    /// #   ehm1::spi::Transaction::async_read(0),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500, Interrupt};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// let ir: Interrupt = w5500.ir().await?;
    /// assert_eq!(ir, Interrupt::default());
    /// # Ok(()) }
    /// ```
    fn ir(&mut self) -> IrFuture<'_, Self> {
        async move {
            let mut reg: [u8; 1] = [0];
            self.read(Reg::IR.addr(), COMMON_BLOCK_OFFSET, &mut reg)
                .await?;
            Ok(Interrupt::from(reg[0]))
        }
    }

    /// Set the interrupt status.
    ///
    /// Setting an interrupt bit to `1` will clear the interrupt.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x15, 0x00]),
    /// #   ehm1::spi::Transaction::async_read(0),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x15, 0x04]),
    /// #   ehm1::spi::Transaction::async_write(0),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500, Interrupt};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// let ir: Interrupt = w5500.ir().await?;
    /// w5500.set_ir(ir).await?;
    /// # Ok(()) }
    /// ```
    fn set_ir(&mut self, interrupt: Interrupt) -> SetIrFuture<'_, Self> {
        async move {
            self.write(Reg::IR.addr(), COMMON_BLOCK_OFFSET, &[interrupt.into()])
                .await
        }
    }

    /// Get the interrupt mask.
    ///
    /// `0` indicates the interrupt is masked.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x16, 0x00]),
    /// #   ehm1::spi::Transaction::async_read(0),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500, Interrupt};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// let imr: Interrupt = w5500.imr().await?;
    /// assert_eq!(imr, Interrupt::default());
    /// # Ok(()) }
    /// ```
    fn imr(&mut self) -> ImrFuture<'_, Self> {
        async move {
            let mut reg: [u8; 1] = [0];
            self.read(Reg::IMR.addr(), COMMON_BLOCK_OFFSET, &mut reg)
                .await?;
            Ok(Interrupt::from(reg[0]))
        }
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
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500, Interrupt};
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x16, 0x04]),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![Interrupt::MP_MASK]),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    ///
    /// // enable the magic packet interrupt
    /// const IMR: Interrupt = Interrupt::DEFAULT.set_mp();
    ///
    /// let mut w5500 = W5500::new(spi);
    /// w5500.set_imr(IMR).await?;
    /// # Ok(()) }
    /// ```
    fn set_imr(&mut self, mask: Interrupt) -> SetImrFuture<'_, Self> {
        async move {
            self.write(Reg::IMR.addr(), COMMON_BLOCK_OFFSET, &[mask.into()])
                .await
        }
    }

    /// Get the socket interrupt status.
    ///
    /// SIMR indicates the interrupt status of all sockets.
    /// Each bit of SIR will be `1` until [`sn_ir`] is cleared.
    /// If [`sn_ir`] is not equal to `0x00` the n<sub>th</sub>
    /// bit of `sir` is `1` and the INTn pin is asserted until SIR is `0x00`.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x17, 0x00]),
    /// #   ehm1::spi::Transaction::async_read(0),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500, SOCKETS};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// let sir = w5500.sir().await?;
    /// // clear all socket interrupts
    /// for socket in SOCKETS.iter() {
    ///     if 1 << (*socket as u8) & sir != 0 {
    ///         let sn_ir = w5500.sn_ir(*socket).await?;
    ///         w5500.set_sn_ir(*socket, sn_ir.into()).await?;
    ///     }
    /// }
    /// # Ok(()) }
    /// ```
    ///
    /// [`sn_ir`]: Registers::sn_ir
    fn sir(&mut self) -> SirFuture<'_, Self> {
        async move {
            let mut reg: [u8; 1] = [0];
            self.read(Reg::SIR.addr(), COMMON_BLOCK_OFFSET, &mut reg)
                .await?;
            Ok(reg[0])
        }
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
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x18, 0x00]),
    /// #   ehm1::spi::Transaction::async_read(0),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// let simr: u8 = w5500.simr().await?;
    /// # Ok(()) }
    /// ```
    fn simr(&mut self) -> SimrFuture<'_, Self> {
        async move {
            let mut reg: [u8; 1] = [0];
            self.read(Reg::SIMR.addr(), COMMON_BLOCK_OFFSET, &mut reg)
                .await?;
            Ok(reg[0])
        }
    }

    /// Set the socket interrupt mask.
    ///
    /// See [`Registers::simr`] for more information.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x18, 0x04]),
    /// #   ehm1::spi::Transaction::async_write(0xFF),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// // enable all socket interrupts
    /// w5500.set_simr(0xFF).await?;
    /// # Ok(()) }
    /// ```
    fn set_simr(&mut self, simr: u8) -> SetSimrFuture<'_, Self> {
        async move {
            self.write(Reg::SIMR.addr(), COMMON_BLOCK_OFFSET, &[simr])
                .await
        }
    }

    /// Get the retry time.
    ///
    /// RTR configures the re-transmission timeout period.
    /// The unit of timeout period is 100 us and the default of RTR is `0x07D0`
    /// or `2000`.
    /// And so the default timeout period is 200 ms (100 us X 2000).
    /// During the time configured by RTR, the W5500 waits for the peer response
    /// to the packet that is transmitted by Sn_CR (CONNECT, DISCON, CLOSE,
    /// SEND, SEND_MAC, SEND_KEEP command).
    /// If the peer does not respond within the RTR time, the W5500 re-transmits
    /// the packet or issues a timeout.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x19, 0x00]),
    /// #   ehm1::spi::Transaction::async_read_vec(vec![0x07, 0xD0]),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// let rtr: u16 = w5500.rtr().await?;
    /// assert_eq!(rtr, 0x07D0);
    /// # Ok(()) }
    /// ```
    fn rtr(&mut self) -> RtrFuture<'_, Self> {
        async move {
            let mut buf: [u8; 2] = [0; 2];
            self.read(Reg::RTR0.addr(), COMMON_BLOCK_OFFSET, &mut buf)
                .await?;
            Ok(u16::from_be_bytes(buf))
        }
    }

    /// Set the retry time.
    ///
    /// See [`Registers::rtr`] for more information.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x19, 0x04]),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x12, 0x34]),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// w5500.set_rtr(0x1234).await?;
    /// # Ok(()) }
    /// ```
    fn set_rtr(&mut self, rtr: u16) -> SetRtrFuture<'_, Self> {
        async move {
            self.write(Reg::RTR0.addr(), COMMON_BLOCK_OFFSET, &rtr.to_be_bytes())
                .await
        }
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
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x1B, 0x00]),
    /// #   ehm1::spi::Transaction::async_read(0x08),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// let rcr: u8 = w5500.rcr().await?;
    /// # Ok(()) }
    /// ```
    fn rcr(&mut self) -> RcrFuture<'_, Self> {
        async move {
            let mut reg: [u8; 1] = [0];
            self.read(Reg::RCR.addr(), COMMON_BLOCK_OFFSET, &mut reg)
                .await?;
            Ok(reg[0])
        }
    }

    /// Set the retry count.
    ///
    /// See [`Registers::rcr`] for more information.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x1B, 0x04]),
    /// #   ehm1::spi::Transaction::async_write(0x0A),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// w5500.set_rcr(0x0A).await?;
    /// # Ok(()) }
    /// ```
    fn set_rcr(&mut self, rcr: u8) -> SetRcrFuture<'_, Self> {
        async move {
            self.write(Reg::RCR.addr(), COMMON_BLOCK_OFFSET, &[rcr])
                .await
        }
    }

    /// Get the PPP link control protocol request timer.
    ///
    /// PTIMER configures the time for sending LCP echo request.
    ///
    /// The unit of time is 25 milliseconds, for a register value of 200 the
    /// timer is 5 seconds.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x1C, 0x00]),
    /// #   ehm1::spi::Transaction::async_read(0x08),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// let ptimer: u8 = w5500.ptimer().await?;
    /// # Ok(()) }
    /// ```
    fn ptimer(&mut self) -> PtimerFuture<'_, Self> {
        async move {
            let mut reg: [u8; 1] = [0];
            self.read(Reg::PTIMER.addr(), COMMON_BLOCK_OFFSET, &mut reg)
                .await?;
            Ok(reg[0])
        }
    }

    /// Set the PPP link control protocol request timer.
    ///
    /// See [`Registers::ptimer`] for more information.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x1C, 0x04]),
    /// #   ehm1::spi::Transaction::async_write(0xC8),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// w5500.set_ptimer(200).await?;
    /// # Ok(()) }
    /// ```
    fn set_ptimer(&mut self, ptimer: u8) -> SetPtimerFuture<'_, Self> {
        async move {
            self.write(Reg::PTIMER.addr(), COMMON_BLOCK_OFFSET, &[ptimer])
                .await
        }
    }

    /// Get the PPP link control protocol magic number.
    ///
    /// PMAGIC configures the 4 byte magic number used in the LCP echo request.
    /// For a register value of `0x01` the magic number is `0x01010101`.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x1D, 0x00]),
    /// #   ehm1::spi::Transaction::async_read(0x08),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// let pmagic: u8 = w5500.pmagic().await?;
    /// # Ok(()) }
    /// ```
    fn pmagic(&mut self) -> PmagicFuture<'_, Self> {
        async move {
            let mut reg: [u8; 1] = [0];
            self.read(Reg::PMAGIC.addr(), COMMON_BLOCK_OFFSET, &mut reg)
                .await?;
            Ok(reg[0])
        }
    }

    /// Set the PPP link control protocol magic number.
    ///
    /// See [`Registers::pmagic`] for more information.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x1D, 0x04]),
    /// #   ehm1::spi::Transaction::async_write(0x01),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// w5500.set_pmagic(0x01).await?;
    /// # Ok(()) }
    /// ```
    fn set_pmagic(&mut self, pmagic: u8) -> SetPmagicFuture<'_, Self> {
        async move {
            self.write(Reg::PMAGIC.addr(), COMMON_BLOCK_OFFSET, &[pmagic])
                .await
        }
    }

    /// Get the destination hardware address in PPPoE mode.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x1E, 0x00]),
    /// #   ehm1::spi::Transaction::async_read_vec(vec![0, 0, 0, 0, 0, 0]),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500, net::Eui48Addr};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// let phar = w5500.phar().await?;
    /// assert_eq!(phar, Eui48Addr::UNSPECIFIED);
    /// # Ok(()) }
    /// ```
    fn phar(&mut self) -> PharFuture<'_, Self> {
        async move {
            let mut phar = Eui48Addr::UNSPECIFIED;
            self.read(Reg::PHAR0.addr(), COMMON_BLOCK_OFFSET, &mut phar.octets)
                .await?;
            Ok(phar)
        }
    }

    /// Set the destination hardware address in PPPoE mode.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x1E, 0x04]),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x12, 0x34, 0, 0, 0, 0]),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500, net::Eui48Addr};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// w5500
    ///     .set_phar(&Eui48Addr::new(0x12, 0x34, 0x00, 0x00, 0x00, 0x00))
    ///     .await?;
    /// # Ok(()) }
    /// ```
    fn set_phar<'a>(&'a mut self, phar: &'a Eui48Addr) -> SetPharFuture<'a, Self> {
        async move {
            self.write(Reg::PHAR0.addr(), COMMON_BLOCK_OFFSET, &phar.octets)
                .await
        }
    }

    /// Get the session ID in PPPoE mode.
    ///
    /// PSID should be written to the PPPoE server session ID acquired in the
    /// PPPoE connection process.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x24, 0x00]),
    /// #   ehm1::spi::Transaction::async_read_vec(vec![0, 0]),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// let psid: u16 = w5500.psid().await?;
    /// assert_eq!(psid, 0x0000);
    /// # Ok(()) }
    /// ```
    fn psid(&mut self) -> PsidFuture<'_, Self> {
        async move {
            let mut buf: [u8; 2] = [0; 2];
            self.read(Reg::PSID0.addr(), COMMON_BLOCK_OFFSET, &mut buf)
                .await?;
            Ok(u16::from_be_bytes(buf))
        }
    }

    /// Set the session ID in PPPoE mode.
    ///
    /// See [`Registers::psid`] for more information.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x24, 0x04]),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x12, 0x34]),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// w5500.set_psid(0x1234).await?;
    /// # Ok(()) }
    /// ```
    fn set_psid(&mut self, psid: u16) -> SetPsidFuture<'_, Self> {
        async move {
            self.write(Reg::PSID0.addr(), COMMON_BLOCK_OFFSET, &psid.to_be_bytes())
                .await
        }
    }

    /// Get the maximum receive unit in PPPoE mode.
    ///
    /// PMRU configures the maximum receive unit of PPPoE.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x26, 0x00]),
    /// #   ehm1::spi::Transaction::async_read_vec(vec![0, 0]),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// let pmru: u16 = w5500.pmru().await?;
    /// assert_eq!(pmru, 0x0000);
    /// # Ok(()) }
    /// ```
    fn pmru(&mut self) -> PmruFuture<'_, Self> {
        async move {
            let mut buf: [u8; 2] = [0; 2];
            self.read(Reg::PMRU0.addr(), COMMON_BLOCK_OFFSET, &mut buf)
                .await?;
            Ok(u16::from_be_bytes(buf))
        }
    }

    /// Set the maximum receive unit in PPPoE mode.
    ///
    /// See [`Registers::pmru`] for more information.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x26, 0x04]),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x12, 0x34]),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// w5500.set_pmru(0x1234).await?;
    /// # Ok(()) }
    /// ```
    fn set_pmru(&mut self, pmru: u16) -> SetPmruFuture<'_, Self> {
        async move {
            self.write(Reg::PMRU0.addr(), COMMON_BLOCK_OFFSET, &pmru.to_be_bytes())
                .await
        }
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
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x28, 0x00]),
    /// #   ehm1::spi::Transaction::async_read_vec(vec![0, 0, 0, 0]),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500, net::Ipv4Addr};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// let uipr = w5500.uipr().await?;
    /// assert_eq!(uipr, Ipv4Addr::UNSPECIFIED);
    /// # Ok(()) }
    /// ```
    fn uipr(&mut self) -> UiprFuture<'_, Self> {
        async move {
            let mut uipr = Ipv4Addr::UNSPECIFIED;
            self.read(Reg::UIPR0.addr(), COMMON_BLOCK_OFFSET, &mut uipr.octets)
                .await?;
            Ok(uipr)
        }
    }

    /// Get the unreachable port.
    ///
    /// See [`Registers::uipr`] for more information.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x2C, 0x00]),
    /// #   ehm1::spi::Transaction::async_read_vec(vec![0, 0]),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// let uportr = w5500.uportr().await?;
    /// # Ok(()) }
    /// ```
    fn uportr(&mut self) -> UportrFuture<'_, Self> {
        async move {
            let mut buf: [u8; 2] = [0; 2];
            self.read(Reg::UPORTR0.addr(), COMMON_BLOCK_OFFSET, &mut buf)
                .await?;
            Ok(u16::from_be_bytes(buf))
        }
    }

    /// Get the PHY configuration.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x2E, 0x00]),
    /// #   ehm1::spi::Transaction::async_read(0b10111000),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500, PhyCfg};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// let phy_cfg: PhyCfg = w5500.phycfgr().await?;
    /// assert_eq!(phy_cfg, PhyCfg::default());
    /// # Ok(()) }
    /// ```
    fn phycfgr(&mut self) -> PhycfgrFuture<'_, Self> {
        async move {
            let mut reg: [u8; 1] = [0];
            self.read(Reg::PHYCFGR.addr(), COMMON_BLOCK_OFFSET, &mut reg)
                .await?;
            Ok(PhyCfg::from(reg[0]))
        }
    }

    /// Set the PHY configuration.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x2E, 0x04]),
    /// #   ehm1::spi::Transaction::async_write(0b11111000),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500, OperationMode, PhyCfg};
    ///
    /// const PHY_CFG: PhyCfg = PhyCfg::DEFAULT.set_opmdc(OperationMode::Auto);
    /// let mut w5500 = W5500::new(spi);
    /// w5500.set_phycfgr(PHY_CFG).await?;
    /// # Ok(()) }
    /// ```
    fn set_phycfgr(&mut self, phycfg: PhyCfg) -> SetPhycfgrFuture<'_, Self> {
        async move {
            self.write(Reg::PHYCFGR.addr(), COMMON_BLOCK_OFFSET, &[phycfg.into()])
                .await
        }
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
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x39, 0x00]),
    /// #   ehm1::spi::Transaction::async_read(0x04),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// let version = w5500.version().await?;
    /// assert_eq!(version, 0x04);
    /// # Ok(()) }
    /// ```
    fn version(&mut self) -> VersionFuture<'_, Self> {
        async move {
            let mut reg: [u8; 1] = [0];
            self.read(Reg::VERSIONR.addr(), COMMON_BLOCK_OFFSET, &mut reg)
                .await?;
            Ok(reg[0])
        }
    }

    /// Get the socket mode.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x00, 0x08]),
    /// #   ehm1::spi::Transaction::async_read(0),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500, Sn, SocketMode};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// let socket_mode = w5500.sn_mr(Sn::Sn0).await?;
    /// assert_eq!(socket_mode, SocketMode::default());
    /// # Ok(()) }
    /// ```
    fn sn_mr(&mut self, sn: Sn) -> SnMrFuture<'_, Self> {
        async move {
            let mut reg: [u8; 1] = [0];
            self.read(SnReg::MR.addr(), sn.block(), &mut reg).await?;
            Ok(SocketMode::from(reg[0]))
        }
    }

    /// Set the socket mode.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x00, 0x08 | 0x04]),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x01]),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500, Protocol, Sn, SocketMode};
    ///
    /// const SOCKET_MODE: SocketMode = SocketMode::DEFAULT.set_protocol(Protocol::Tcp);
    /// let mut w5500 = W5500::new(spi);
    /// w5500.set_sn_mr(Sn::Sn0, SOCKET_MODE).await?;
    /// # Ok(()) }
    /// ```
    fn set_sn_mr(&mut self, sn: Sn, mode: SocketMode) -> SetSnMrFuture<'_, Self> {
        async move {
            self.write(SnReg::MR.addr(), sn.block(), &[mode.into()])
                .await
        }
    }

    /// Get the socket command.
    ///
    /// The only use for reading this register is to check if a socket command
    /// has been accepted.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x01, 0x08 | 0x04]),
    /// #   ehm1::spi::Transaction::async_write(0x01),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x01, 0x08]),
    /// #   ehm1::spi::Transaction::async_read(1),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x01, 0x08]),
    /// #   ehm1::spi::Transaction::async_read(0),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500, Sn, SocketCommand};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// w5500.set_sn_cr(Sn::Sn0, SocketCommand::Open).await?;
    /// while w5500.sn_cr(Sn::Sn0).await? != SocketCommand::Accepted.into() {}
    /// # Ok(()) }
    /// ```
    fn sn_cr(&mut self, sn: Sn) -> SnCrFuture<'_, Self> {
        async move {
            let mut reg: [u8; 1] = [0];
            self.read(SnReg::CR.addr(), sn.block(), &mut reg).await?;
            Ok(reg[0])
        }
    }

    /// Set the socket command.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x01, 0x08 | 0x04]),
    /// #   ehm1::spi::Transaction::async_write(0x01),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500, Sn, SocketCommand};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// w5500.set_sn_cr(Sn::Sn0, SocketCommand::Open).await?;
    /// # Ok(()) }
    /// ```
    fn set_sn_cr(&mut self, sn: Sn, cmd: SocketCommand) -> SetSnCrFuture<'_, Self> {
        async move {
            self.write(SnReg::CR.addr(), sn.block(), &[cmd.into()])
                .await
        }
    }

    /// Get the socket interrupt status.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x02, 0x08]),
    /// #   ehm1::spi::Transaction::async_read(0),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500, Sn};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// let socket_interrupts = w5500.sn_ir(Sn::Sn0).await?;
    /// # Ok(()) }
    /// ```
    fn sn_ir(&mut self, sn: Sn) -> SnIrFuture<'_, Self> {
        async move {
            let mut reg: [u8; 1] = [0];
            self.read(SnReg::IR.addr(), sn.block(), &mut reg).await?;
            Ok(SocketInterrupt::from(reg[0]))
        }
    }

    /// Set the socket interrupt status.
    ///
    /// This is a write 1 to clear register.
    ///
    /// # Examples
    ///
    /// Clearing all raised interrupts.
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x02, 0x08]),
    /// #   ehm1::spi::Transaction::async_read(0),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x02, 0x08 | 0x04]),
    /// #   ehm1::spi::Transaction::async_write(0),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500, Sn, SocketInterrupt};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// let socket_interrupts: SocketInterrupt = w5500.sn_ir(Sn::Sn0).await?;
    /// w5500.set_sn_ir(Sn::Sn0, socket_interrupts.into()).await?;
    /// # Ok(()) }
    /// ```
    ///
    /// Clearing only the SENDOK interrupt.
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x02, 0x08 | 0x04]),
    /// #   ehm1::spi::Transaction::async_write(SocketInterrupt::SENDOK_MASK),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500, Sn, SocketInterrupt};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// w5500
    ///     .set_sn_ir(Sn::Sn0, SocketInterrupt::SENDOK_MASK)
    ///     .await?;
    /// # Ok(()) }
    /// ```
    fn set_sn_ir(&mut self, sn: Sn, sn_ir: u8) -> SetSnIrFuture<'_, Self> {
        // todo: not consistent with synchronous version
        // https://github.com/rust-lang/rust/issues/65442
        async move { self.write(SnReg::IR.addr(), sn.block(), &[sn_ir]).await }
    }

    /// Get the socket status.
    ///
    /// **Note:** This method returns a nested [`core::result::Result`].
    ///
    /// The outermost `Result` is for handling bus errors, similar to most of
    /// the other methods in this trait.
    ///
    /// The innermost `Result<SocketStatus, u8>` is the result of a `u8` to
    /// [`SocketStatus`] conversion because not every value of `u8` corresponds
    /// to a valid [`SocketStatus`].
    /// * `u8` values that have a corresponding [`SocketStatus`] will be
    ///   converted and returned in the [`Ok`] variant of the inner `Result`.
    /// * `u8` values that do not corresponding [`SocketStatus`] will have the
    ///   raw `u8` byte returned in the [`Err`] variant of the inner `Result`.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x03, 0x08]),
    /// #   ehm1::spi::Transaction::async_read(0),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500, Sn, SocketStatus};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// let sn_sr = w5500.sn_sr(Sn::Sn0).await?;
    /// assert_eq!(sn_sr, Ok(SocketStatus::Closed));
    /// # Ok(()) }
    /// ```
    ///
    /// [`Ok`]: https://doc.rust-lang.org/core/result/enum.Result.html#variant.Ok
    /// [`Err`]: https://doc.rust-lang.org/core/result/enum.Result.html#variant.Err
    fn sn_sr(&mut self, sn: Sn) -> SnSrFuture<'_, Self> {
        async move {
            let mut reg: [u8; 1] = [0];
            self.read(SnReg::SR.addr(), sn.block(), &mut reg).await?;
            Ok(SocketStatus::try_from(reg[0]))
        }
    }

    /// Get the socket source port.
    ///
    /// This is only valid in TCP/UDP mode.
    /// This should be set before sending the OPEN command.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x04, 0x08]),
    /// #   ehm1::spi::Transaction::async_read_vec(vec![0, 0]),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500, Sn, SocketMode};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// let socket_port: u16 = w5500.sn_port(Sn::Sn0).await?;
    /// # Ok(()) }
    /// ```
    fn sn_port(&mut self, sn: Sn) -> SnPortFuture<'_, Self> {
        async move {
            let mut reg: [u8; 2] = [0; 2];
            self.read(SnReg::PORT0.addr(), sn.block(), &mut reg).await?;
            Ok(u16::from_be_bytes(reg))
        }
    }

    /// Set the socket source port.
    ///
    /// See [`Registers::sn_port`] for more information.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x04, 0x08 | 0x04]),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 68]),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500, Sn};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// w5500.set_sn_port(Sn::Sn0, 68).await?;
    /// # Ok(()) }
    /// ```
    fn set_sn_port(&mut self, sn: Sn, port: u16) -> SetSnPortFuture<'_, Self> {
        async move {
            self.write(SnReg::PORT0.addr(), sn.block(), &u16::to_be_bytes(port))
                .await
        }
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
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x06, 0x08]),
    /// #   ehm1::spi::Transaction::async_read_vec(vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500, Sn};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// let dhar = w5500.sn_dhar(Sn::Sn0).await?;
    /// # Ok(()) }
    /// ```
    fn sn_dhar(&mut self, sn: Sn) -> SnDharFuture<'_, Self> {
        async move {
            let mut dhar: Eui48Addr = Eui48Addr::UNSPECIFIED;
            self.read(SnReg::DHAR0.addr(), sn.block(), &mut dhar.octets)
                .await?;
            Ok(dhar)
        }
    }

    /// Set the socket destination hardware address.
    ///
    /// See [`Registers::sn_dhar`] for more information.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x06, 0x08 | 0x04]),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x12, 0x34, 0x00, 0x00, 0x00, 0x00]),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500, net::Eui48Addr, Sn};
    ///
    /// let dhar = Eui48Addr::new(0x12, 0x34, 0x00, 0x00, 0x00, 0x00);
    /// let mut w5500 = W5500::new(spi);
    /// w5500.set_sn_dhar(Sn::Sn0, &dhar).await?;
    /// # Ok(()) }
    /// ```
    fn set_sn_dhar<'a>(&'a mut self, sn: Sn, dhar: &'a Eui48Addr) -> SetSnDharFuture<'a, Self> {
        async move {
            self.write(SnReg::DHAR0.addr(), sn.block(), &dhar.octets)
                .await
        }
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
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x0C, 0x08]),
    /// #   ehm1::spi::Transaction::async_read_vec(vec![0, 0, 0, 0]),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500, net::Ipv4Addr, Sn};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// let dipr = w5500.sn_dipr(Sn::Sn0).await?;
    /// assert_eq!(dipr, Ipv4Addr::UNSPECIFIED);
    /// # Ok(()) }
    /// ```
    fn sn_dipr(&mut self, sn: Sn) -> SnDiprFuture<'_, Self> {
        async move {
            let mut dipr: Ipv4Addr = Ipv4Addr::UNSPECIFIED;
            self.read(SnReg::DIPR0.addr(), sn.block(), &mut dipr.octets)
                .await?;
            Ok(dipr)
        }
    }

    /// Set the socket destination IP address.
    ///
    /// See [`Registers::sn_dipr`] for more information.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x0C, 0x08 | 0x04]),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![192, 168, 0, 11]),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500, net::Ipv4Addr, Sn};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// w5500
    ///     .set_sn_dipr(Sn::Sn0, &Ipv4Addr::new(192, 168, 0, 11))
    ///     .await?;
    /// # Ok(()) }
    /// ```
    fn set_sn_dipr<'a>(&'a mut self, sn: Sn, dipr: &'a Ipv4Addr) -> SetSnDiprFuture<'a, Self> {
        async move {
            self.write(SnReg::DIPR0.addr(), sn.block(), &dipr.octets)
                .await
        }
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
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x10, 0x08]),
    /// #   ehm1::spi::Transaction::async_read_vec(vec![0, 0]),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500, Sn, SocketMode};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// let socket_destination_port: u16 = w5500.sn_dport(Sn::Sn0).await?;
    /// # Ok(()) }
    /// ```
    fn sn_dport(&mut self, sn: Sn) -> SnDportFuture<'_, Self> {
        async move {
            let mut reg: [u8; 2] = [0; 2];
            self.read(SnReg::DPORT0.addr(), sn.block(), &mut reg)
                .await?;
            Ok(u16::from_be_bytes(reg))
        }
    }

    /// Set the socket destination port.
    ///
    /// See [`Registers::sn_dport`] for more information.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x10, 0x08 | 0x04]),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 67]),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500, Sn};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// w5500.set_sn_dport(Sn::Sn0, 67).await?;
    /// # Ok(()) }
    /// ```
    fn set_sn_dport(&mut self, sn: Sn, port: u16) -> SetSnDportFuture<'_, Self> {
        async move {
            self.write(SnReg::DPORT0.addr(), sn.block(), &u16::to_be_bytes(port))
                .await
        }
    }

    /// Get the socket destination IPv4 and port.
    ///
    /// This is a compound which performs [`Registers::sn_dipr`] and
    /// [`Registers::sn_dport`] together.
    ///
    /// The `sn_dipr` and `sn_dport` registers are contiguous in memory, which
    /// allows this function to do one read transfer to read both registers.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x0C, 0x08]),
    /// #   ehm1::spi::Transaction::async_read_vec(vec![0, 0, 0, 0, 0, 0]),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500, net::SocketAddrV4, Sn};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// let addr = w5500.sn_dest(Sn::Sn0).await?;
    /// assert_eq!(addr, SocketAddrV4::default());
    /// # Ok(()) }
    /// ```
    fn sn_dest(&mut self, sn: Sn) -> SnDestFuture<'_, Self> {
        async move {
            let mut buf: [u8; 6] = [0; 6];
            self.read(SnReg::DIPR0.addr(), sn.block(), &mut buf).await?;
            Ok(SocketAddrV4::new(
                Ipv4Addr::new(buf[0], buf[1], buf[2], buf[3]),
                u16::from_be_bytes([buf[4], buf[5]]),
            ))
        }
    }

    /// Set the socket destination IPv4 and port.
    ///
    /// This is a compound operation which performs
    /// [`Registers::set_sn_dipr`] and [`Registers::set_sn_dport`] together.
    ///
    /// The `sn_dipr` and `sn_dport` registers are contiguous in memory, which
    /// allows this function to do one write transfer to write both registers.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x0C, 0x08 | 0x04]),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![192, 168, 0, 11, 0, 67]),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{
    ///     aio::Registers,
    ///     eh1::vdm::W5500,
    ///     net::{Ipv4Addr, SocketAddrV4},
    ///     Sn,
    /// };
    ///
    /// let addr: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(192, 168, 0, 11), 67);
    /// let mut w5500 = W5500::new(spi);
    /// w5500.set_sn_dest(Sn::Sn0, &addr).await?;
    /// # Ok(()) }
    /// ```
    fn set_sn_dest<'a>(&'a mut self, sn: Sn, addr: &'a SocketAddrV4) -> SetSnDestFuture<'a, Self> {
        async move {
            let buf: [u8; 6] = [
                addr.ip().octets[0],
                addr.ip().octets[1],
                addr.ip().octets[2],
                addr.ip().octets[3],
                (addr.port() >> 8) as u8,
                addr.port() as u8,
            ];
            self.write(SnReg::DIPR0.addr(), sn.block(), &buf).await
        }
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
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x12, 0x08]),
    /// #   ehm1::spi::Transaction::async_read_vec(vec![0x00, 0x00]),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500, Sn};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// let sn__mssr: u16 = w5500.sn_mssr(Sn::Sn0).await?;
    /// # Ok(()) }
    /// ```
    fn sn_mssr(&mut self, sn: Sn) -> SnMssrFuture<'_, Self> {
        async move {
            let mut reg: [u8; 2] = [0; 2];
            self.read(SnReg::MSSR0.addr(), sn.block(), &mut reg).await?;
            Ok(u16::from_be_bytes(reg))
        }
    }

    /// Set the socket maximum segment size.
    ///
    /// See [`Registers::sn_mssr`] for lots more information.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x12, 0x08 | 0x04]),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x05, 0xB4]),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500, Sn};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// w5500.set_sn_mssr(Sn::Sn0, 1460).await?;
    /// # Ok(()) }
    /// ```
    fn set_sn_mssr(&mut self, sn: Sn, mssr: u16) -> SetSnMssrFuture<'_, Self> {
        async move {
            self.write(SnReg::MSSR0.addr(), sn.block(), &u16::to_be_bytes(mssr))
                .await
        }
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
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x15, 0x08]),
    /// #   ehm1::spi::Transaction::async_read(0),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500, Sn};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// let tos: u8 = w5500.sn_tos(Sn::Sn0).await?;
    /// # Ok(()) }
    /// ```
    fn sn_tos(&mut self, sn: Sn) -> SnTosFuture<'_, Self> {
        async move {
            let mut reg: [u8; 1] = [0];
            self.read(SnReg::TOS.addr(), sn.block(), &mut reg).await?;
            Ok(reg[0])
        }
    }

    /// Set the IP type of service.
    ///
    /// For more information see [`Registers::sn_tos`].
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x15, 0x08 | 0x04]),
    /// #   ehm1::spi::Transaction::async_write(0x01),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500, Sn};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// w5500.set_sn_tos(Sn::Sn0, 1).await?;
    /// # Ok(()) }
    /// ```
    fn set_sn_tos(&mut self, sn: Sn, tos: u8) -> SetSnTosFuture<'_, Self> {
        async move { self.write(SnReg::TOS.addr(), sn.block(), &[tos]).await }
    }

    /// Get the time to live.
    ///
    /// This register configures the TTL (Time to Live field in the IP header)
    /// for socket n.
    ///
    /// For more details see <https://www.iana.org/assignments/ip-parameters>.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x16, 0x08]),
    /// #   ehm1::spi::Transaction::async_read(0x80),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500, Sn};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// let ttl: u8 = w5500.sn_ttl(Sn::Sn0).await?;
    /// # Ok(()) }
    /// ```
    fn sn_ttl(&mut self, sn: Sn) -> SnTtlFuture<'_, Self> {
        async move {
            let mut reg: [u8; 1] = [0];
            self.read(SnReg::TTL.addr(), sn.block(), &mut reg).await?;
            Ok(reg[0])
        }
    }

    /// Set the time to live.
    ///
    /// For more information see [`Registers::sn_ttl`].
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x16, 0x08 | 0x04]),
    /// #   ehm1::spi::Transaction::async_write(0x80),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500, Sn};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// w5500.set_sn_ttl(Sn::Sn0, 0x80).await?;
    /// # Ok(()) }
    /// ```
    fn set_sn_ttl(&mut self, sn: Sn, ttl: u8) -> SetSnTtlFuture<'_, Self> {
        async move { self.write(SnReg::TTL.addr(), sn.block(), &[ttl]).await }
    }

    /// Get the socket RX buffer size.
    ///
    /// The buffer size can be configured to any of the sizes in [`BufferSize`].
    ///
    /// The sum of all the socket RX buffers cannot exceed 16 KiB.
    ///
    /// **Note:** This method returns a nested [`core::result::Result`].
    ///
    /// The outermost `Result` is for handling bus errors, similar to most of
    /// the other methods in this trait.
    ///
    /// The innermost `Result<BufferSize, u8>` is the result of a `u8` to
    /// [`BufferSize`] conversion because not every value of `u8` corresponds
    /// to a valid [`BufferSize`].
    /// * `u8` values that have a corresponding [`BufferSize`] will be
    ///   converted and returned in the [`Ok`] variant of the inner `Result`.
    /// * `u8` values that do not corresponding [`BufferSize`] will have the
    ///   raw `u8` byte returned in the [`Err`] variant of the inner `Result`.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x1E, 0x08]),
    /// #   ehm1::spi::Transaction::async_read(0x02),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500, BufferSize, Sn};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// let sn_rxbuf_size = w5500.sn_rxbuf_size(Sn::Sn0).await?;
    /// assert_eq!(sn_rxbuf_size, Ok(BufferSize::KB2));
    /// # Ok(()) }
    /// ```
    ///
    /// [`Ok`]: https://doc.rust-lang.org/core/result/enum.Result.html#variant.Ok
    /// [`Err`]: https://doc.rust-lang.org/core/result/enum.Result.html#variant.Err
    fn sn_rxbuf_size(&mut self, sn: Sn) -> SnRxbufSizeFuture<'_, Self> {
        async move {
            let mut reg: [u8; 1] = [0];
            self.read(SnReg::RXBUF_SIZE.addr(), sn.block(), &mut reg)
                .await?;
            Ok(BufferSize::try_from(reg[0]))
        }
    }

    /// Set the socket RX buffer size.
    ///
    /// See [`Registers::sn_rxbuf_size`] for more information.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x1E, 0x08 | 0x04]),
    /// #   ehm1::spi::Transaction::async_write(1),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500, BufferSize, Sn};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// w5500.set_sn_rxbuf_size(Sn::Sn0, BufferSize::KB1).await?;
    /// # Ok(()) }
    /// ```
    fn set_sn_rxbuf_size(&mut self, sn: Sn, size: BufferSize) -> SetSnRxbufSizeFuture<'_, Self> {
        async move {
            self.write(SnReg::RXBUF_SIZE.addr(), sn.block(), &[size.into()])
                .await
        }
    }

    /// Get the socket TX buffer size.
    ///
    /// The buffer size can be configured to any of the sizes in [`BufferSize`].
    ///
    /// The sum of all the socket TX buffers cannot exceed 16 KiB.
    ///
    /// **Note:** This method returns a nested [`core::result::Result`].
    ///
    /// The outermost `Result` is for handling bus errors, similar to most of
    /// the other methods in this trait.
    ///
    /// The innermost `Result<BufferSize, u8>` is the result of a `u8` to
    /// [`BufferSize`] conversion because not every value of `u8` corresponds
    /// to a valid [`BufferSize`].
    /// * `u8` values that have a corresponding [`BufferSize`] will be
    ///   converted and returned in the [`Ok`] variant of the inner `Result`.
    /// * `u8` values that do not corresponding [`BufferSize`] will have the
    ///   raw `u8` byte returned in the [`Err`] variant of the inner `Result`.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x1F, 0x08]),
    /// #   ehm1::spi::Transaction::async_read(0x02),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500, BufferSize, Sn};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// let sn_txbuf_size = w5500.sn_txbuf_size(Sn::Sn0).await?;
    /// assert_eq!(sn_txbuf_size, Ok(BufferSize::KB2));
    /// # Ok(()) }
    /// ```
    ///
    /// [`Ok`]: https://doc.rust-lang.org/core/result/enum.Result.html#variant.Ok
    /// [`Err`]: https://doc.rust-lang.org/core/result/enum.Result.html#variant.Err
    fn sn_txbuf_size(&mut self, sn: Sn) -> SnTxbufSizeFuture<'_, Self> {
        async move {
            let mut reg: [u8; 1] = [0];
            self.read(SnReg::TXBUF_SIZE.addr(), sn.block(), &mut reg)
                .await?;
            Ok(BufferSize::try_from(reg[0]))
        }
    }

    /// Set the socket TX buffer size.
    ///
    /// See [`Registers::sn_rxbuf_size`] for more information.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x1F, 0x08 | 0x04]),
    /// #   ehm1::spi::Transaction::async_write(1),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500, BufferSize, Sn};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// w5500.set_sn_txbuf_size(Sn::Sn0, BufferSize::KB1).await?;
    /// # Ok(()) }
    /// ```
    fn set_sn_txbuf_size(&mut self, sn: Sn, size: BufferSize) -> SetSnTxbufSizeFuture<'_, Self> {
        async move {
            self.write(SnReg::TXBUF_SIZE.addr(), sn.block(), &[size.into()])
                .await
        }
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
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x20, 0x08]),
    /// #   ehm1::spi::Transaction::async_read_vec(vec![0x08, 0x00]),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500, Sn, SocketMode};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// let sn_tx_fsr: u16 = w5500.sn_tx_fsr(Sn::Sn0).await?;
    /// # Ok(()) }
    /// ```
    fn sn_tx_fsr(&mut self, sn: Sn) -> SnTxFsrFuture<'_, Self> {
        async move {
            let mut reg: [u8; 2] = [0; 2];
            self.read(SnReg::TX_FSR0.addr(), sn.block(), &mut reg)
                .await?;
            Ok(u16::from_be_bytes(reg))
        }
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
    /// If its increment value exceeds the maximum value `0xFFFF`,
    /// (greater than `0x10000` and the carry bit occurs), then the carry bit is
    /// ignored and will automatically update with the lower 16 bits value.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x22, 0x08]),
    /// #   ehm1::spi::Transaction::async_read_vec(vec![0, 0]),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500, Sn, SocketMode};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// let sn_tx_rd: u16 = w5500.sn_tx_rd(Sn::Sn0).await?;
    /// # Ok(()) }
    /// ```
    fn sn_tx_rd(&mut self, sn: Sn) -> SnTxRdFuture<'_, Self> {
        async move {
            let mut reg: [u8; 2] = [0; 2];
            self.read(SnReg::TX_RD0.addr(), sn.block(), &mut reg)
                .await?;
            Ok(u16::from_be_bytes(reg))
        }
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
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x24, 0x08]),
    /// #   ehm1::spi::Transaction::async_read_vec(vec![0, 0]),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500, Sn};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// let sn_tx_wr: u16 = w5500.sn_tx_wr(Sn::Sn0).await?;
    /// # Ok(()) }
    /// ```
    fn sn_tx_wr(&mut self, sn: Sn) -> SnTxWrFuture<'_, Self> {
        async move {
            let mut reg: [u8; 2] = [0; 2];
            self.read(SnReg::TX_WR0.addr(), sn.block(), &mut reg)
                .await?;
            Ok(u16::from_be_bytes(reg))
        }
    }

    /// Set the socket TX write pointer.
    ///
    /// See [`Registers::sn_tx_wr`] for more information.
    ///
    /// See [`Registers::set_sn_tx_buf`] for an example.
    fn set_sn_tx_wr(&mut self, sn: Sn, ptr: u16) -> SetSnTxWrFuture<'_, Self> {
        async move {
            self.write(SnReg::TX_WR0.addr(), sn.block(), &ptr.to_be_bytes())
                .await
        }
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
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x26, 0x08]),
    /// #   ehm1::spi::Transaction::async_read_vec(vec![0, 0]),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500, Sn};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// let sn_rx_rsr: u16 = w5500.sn_rx_rsr(Sn::Sn0).await?;
    /// # Ok(()) }
    /// ```
    fn sn_rx_rsr(&mut self, sn: Sn) -> SnRxRsrFuture<'_, Self> {
        async move {
            let mut reg: [u8; 2] = [0; 2];
            self.read(SnReg::RX_RSR0.addr(), sn.block(), &mut reg)
                .await?;
            Ok(u16::from_be_bytes(reg))
        }
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
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x28, 0x08]),
    /// #   ehm1::spi::Transaction::async_read_vec(vec![0, 0]),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500, Sn};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// let sn_rx_rd: u16 = w5500.sn_rx_rd(Sn::Sn0).await?;
    /// # Ok(()) }
    /// ```
    fn sn_rx_rd(&mut self, sn: Sn) -> SnRxRdFuture<'_, Self> {
        async move {
            let mut reg: [u8; 2] = [0; 2];
            self.read(SnReg::RX_RD0.addr(), sn.block(), &mut reg)
                .await?;
            Ok(u16::from_be_bytes(reg))
        }
    }

    /// Set the socket read data pointer.
    ///
    /// See [`Registers::sn_rx_rd`] for more information.
    /// See [`Registers::sn_rx_buf`] for an example.
    fn set_sn_rx_rd(&mut self, sn: Sn, ptr: u16) -> SetSnRxRdFuture<'_, Self> {
        async move {
            self.write(SnReg::RX_RD0.addr(), sn.block(), &ptr.to_be_bytes())
                .await
        }
    }

    /// Get the socket RX write pointer.
    ///
    /// This register is initialized by the [`SocketCommand::Open`] command, and
    /// it is auto-incremented by hardware.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x2A, 0x08]),
    /// #   ehm1::spi::Transaction::async_read_vec(vec![0, 0]),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500, Sn};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// let sn_rx_wr: u16 = w5500.sn_rx_wr(Sn::Sn0).await?;
    /// # Ok(()) }
    /// ```
    fn sn_rx_wr(&mut self, sn: Sn) -> SnRxWrFuture<'_, Self> {
        async move {
            let mut reg: [u8; 2] = [0; 2];
            self.read(SnReg::RX_WR0.addr(), sn.block(), &mut reg)
                .await?;
            Ok(u16::from_be_bytes(reg))
        }
    }

    /// Get the socket interrupt mask.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x2C, 0x08]),
    /// #   ehm1::spi::Transaction::async_read(0xFF),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500, Sn, SocketInterruptMask};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// let sn_imr: SocketInterruptMask = w5500.sn_imr(Sn::Sn0).await?;
    /// assert_eq!(sn_imr, SocketInterruptMask::default());
    /// # Ok(()) }
    /// ```
    fn sn_imr(&mut self, sn: Sn) -> SnImrFuture<'_, Self> {
        async move {
            let mut reg: [u8; 1] = [0];
            self.read(SnReg::IMR.addr(), sn.block(), &mut reg).await?;
            Ok(SocketInterruptMask::from(reg[0]))
        }
    }

    /// Set the socket interrupt mask.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x2C, 0x08 | 0x04]),
    /// #   ehm1::spi::Transaction::async_write(0xE0),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500, Sn, SocketInterruptMask};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// w5500
    ///     .set_sn_imr(Sn::Sn0, SocketInterruptMask::ALL_MASKED)
    ///     .await?;
    /// # Ok(()) }
    /// ```
    fn set_sn_imr(&mut self, sn: Sn, mask: SocketInterruptMask) -> SetSnImrFuture<'_, Self> {
        async move {
            self.write(SnReg::IMR.addr(), sn.block(), &[mask.into()])
                .await
        }
    }

    /// Get the socket fragment.
    ///
    /// This configures the fragment field in the IP header.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x2D, 0x08]),
    /// #   ehm1::spi::Transaction::async_read_vec(vec![0x40, 0x00]),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500, Sn};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// let frag: u16 = w5500.sn_frag(Sn::Sn0).await?;
    /// assert_eq!(frag, 0x4000);
    /// # Ok(()) }
    /// ```
    fn sn_frag(&mut self, sn: Sn) -> SnFragFuture<'_, Self> {
        async move {
            let mut buf: [u8; 2] = [0; 2];
            self.read(SnReg::FRAG0.addr(), sn.block(), &mut buf).await?;
            Ok(u16::from_be_bytes(buf))
        }
    }

    /// Set the socket fragment.
    ///
    /// See [`Registers::sn_frag`] for more information.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x2D, 0x08 | 0x04]),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x00]),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500, Sn};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// w5500.set_sn_frag(Sn::Sn0, 0).await?;
    /// # Ok(()) }
    /// ```
    fn set_sn_frag(&mut self, sn: Sn, frag: u16) -> SetSnFragFuture<'_, Self> {
        async move {
            self.write(SnReg::FRAG0.addr(), sn.block(), &u16::to_be_bytes(frag))
                .await
        }
    }

    /// Get the socket keep alive time.
    ///
    /// This register configures the transmitting timer of the keep alive (KA)
    /// packet for the socket.  This is valid only in TCP mode, and is ignored
    /// in all other modes.
    ///
    /// The time unit is 5 seconds.
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
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x2F, 0x08]),
    /// #   ehm1::spi::Transaction::async_read(0),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500, Sn};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// let sn_kpalvtr: u8 = w5500.sn_kpalvtr(Sn::Sn0).await?;
    /// # Ok(()) }
    /// ```
    fn sn_kpalvtr(&mut self, sn: Sn) -> SnKpalvtrFuture<'_, Self> {
        async move {
            let mut buf: [u8; 1] = [0];
            self.read(SnReg::KPALVTR.addr(), sn.block(), &mut buf)
                .await?;
            Ok(buf[0])
        }
    }

    /// Set the socket keep alive time.
    ///
    /// See [`Registers::sn_kpalvtr`] for more information.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x2F, 0x08 | 0x04]),
    /// #   ehm1::spi::Transaction::async_write(0x0A),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// use w5500_ll::{aio::Registers, eh1::vdm::W5500, Sn};
    ///
    /// let mut w5500 = W5500::new(spi);
    /// // 50s keep alive timer
    /// w5500.set_sn_kpalvtr(Sn::Sn0, 10).await?;
    /// # Ok(()) }
    /// ```
    fn set_sn_kpalvtr(&mut self, sn: Sn, kpalvtr: u8) -> SetSnKpalvtrFuture<'_, Self> {
        async move {
            self.write(SnReg::KPALVTR.addr(), sn.block(), &[kpalvtr])
                .await
        }
    }

    /// Write the socket TX buffer.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// use core::cmp::min;
    /// use w5500_ll::{eh1::vdm::W5500, aio::Registers, Sn, SocketCommand};
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   // sn_tx_fsr
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, w5500_ll::SnReg::TX_FSR0.addr() as u8, 0x08]),
    /// #   ehm1::spi::Transaction::async_read_vec(vec![0x08, 0x00]),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// #   // sn_tx_write
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, w5500_ll::SnReg::TX_WR0.addr() as u8, 0x08]),
    /// #   ehm1::spi::Transaction::async_read_vec(vec![0, 0]),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// #   // set_sn_tx_buf
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x00, (Sn::Sn0.tx_block() as u8) << 3 | 0x04]),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x12, 0x34, 0x56, 0x78, 0x9A]),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// #   // set_sn_tx_wr
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x24, 0x08 | 0x04]),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 5]),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// #   // set_sn_cr
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x01, 0x08 | 0x04]),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![SocketCommand::Send.into()]),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// # let mut w5500 = W5500::new(spi);
    ///
    /// // the socket should already be opened at this point
    /// const THE_SOCKET: Sn = Sn::Sn0;
    ///
    /// let buf: [u8; 5] = [0x12, 0x34, 0x56, 0x78, 0x9A];
    ///
    /// // transmit as many bytes as possible
    /// // for large buffers this may not transmit all the available data
    /// let tx_bytes: u16 = {
    ///     min(w5500.sn_tx_fsr(THE_SOCKET).await?, u16::try_from(buf.len()).unwrap_or(u16::MAX))
    /// };
    /// if tx_bytes == 0 {
    ///     return Ok(());
    /// }
    ///
    /// let ptr: u16 = w5500.sn_tx_wr(THE_SOCKET).await?;
    /// w5500.set_sn_tx_buf(THE_SOCKET, ptr, &buf[..usize::from(tx_bytes)]).await?;
    /// w5500.set_sn_tx_wr(THE_SOCKET, ptr.wrapping_add(tx_bytes)).await?;
    /// w5500.set_sn_cr(THE_SOCKET, SocketCommand::Send).await?;
    /// # Ok(()) }
    /// ```
    fn set_sn_tx_buf<'a>(
        &'a mut self,
        sn: Sn,
        ptr: u16,
        buf: &'a [u8],
    ) -> SetSnTxBufFuture<'a, Self> {
        async move { self.write(ptr, sn.tx_block(), buf).await }
    }

    /// Read the socket TX buffer.
    ///
    /// This method is typically unused; there are very few use cases that
    /// require reading the TX buffer.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// use core::cmp::min;
    /// use w5500_ll::{eh1::vdm::W5500, aio::Registers, Sn, SocketCommand};
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x00, (Sn::Sn0.tx_block() as u8) << 3]),
    /// #   ehm1::spi::Transaction::async_read(0),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// # let mut w5500 = W5500::new(spi);
    /// let mut buf: [u8; 1] = [0];
    /// w5500.sn_tx_buf(Sn::Sn0, 0, &mut buf).await?;
    /// # Ok(()) }
    /// ```
    fn sn_tx_buf<'a>(&'a mut self, sn: Sn, ptr: u16, buf: &'a mut [u8]) -> SnTxBufFuture<'a, Self> {
        async move { self.read(ptr, sn.tx_block(), buf).await }
    }

    /// Read the socket RX buffer.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// use core::cmp::min;
    /// use w5500_ll::{eh1::vdm::W5500, aio::Registers, Sn, SocketCommand};
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, w5500_ll::SnReg::RX_RSR0.addr() as u8, 0x08]),
    /// #   ehm1::spi::Transaction::async_read_vec(vec![0, 4]),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, w5500_ll::SnReg::RX_RD0.addr() as u8, 0x08]),
    /// #   ehm1::spi::Transaction::async_read_vec(vec![0, 0]),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x00, (Sn::Sn0.rx_block() as u8) << 3]),
    /// #   ehm1::spi::Transaction::async_read_vec(vec![0, 0, 0, 0]),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, w5500_ll::SnReg::RX_RD0.addr() as u8, 0x08 | 0x04]),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0, 4]),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, w5500_ll::SnReg::CR.addr() as u8, 0x08 | 0x04]),
    /// #   ehm1::spi::Transaction::async_write(SocketCommand::Recv.into()),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// # let mut w5500 = W5500::new(spi);
    ///
    /// // the socket should already be opened at this point
    /// // a socket interrupt will indicate there is data to be retrieved
    /// const THE_SOCKET: Sn = Sn::Sn0;
    ///
    /// // in reality you will need a larger buffer for most protocols
    /// const BUF_LEN: usize = 16;
    /// let mut buf: [u8; BUF_LEN] = [0; BUF_LEN];
    ///
    /// let rx_bytes: u16 = {
    ///     min(w5500.sn_rx_rsr(THE_SOCKET).await?, u16::try_from(buf.len()).unwrap_or(u16::MAX))
    /// };
    /// if rx_bytes == 0 {
    ///     return Ok(());
    /// }
    ///
    /// let ptr: u16 = w5500.sn_rx_rd(THE_SOCKET).await?;
    /// w5500.sn_rx_buf(THE_SOCKET, ptr, &mut buf[..usize::from(rx_bytes)]).await?;
    /// w5500.set_sn_rx_rd(THE_SOCKET, ptr.wrapping_add(rx_bytes)).await?;
    /// w5500.set_sn_cr(THE_SOCKET, SocketCommand::Recv).await?;
    /// # Ok(()) }
    /// ```
    fn sn_rx_buf<'a>(&'a mut self, sn: Sn, ptr: u16, buf: &'a mut [u8]) -> SnRxBufFuture<'a, Self> {
        async move { self.read(ptr, sn.rx_block(), buf).await }
    }

    /// Write the socket RX buffer.
    ///
    /// This method is typically unused; there are very few use cases that
    /// require writing the RX buffer.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn dox() -> Result<(), eh1::spi::ErrorKind> {
    /// use core::cmp::min;
    /// use w5500_ll::{eh1::vdm::W5500, aio::Registers, Sn, SocketCommand};
    /// # let spi = ehm1::spi::Mock::new(&[
    /// #   ehm1::spi::Transaction::async_transaction_start(),
    /// #   ehm1::spi::Transaction::async_write_vec(vec![0x00, 0x00, (Sn::Sn0.rx_block() as u8) << 3 | 0x04]),
    /// #   ehm1::spi::Transaction::async_write(0),
    /// #   ehm1::spi::Transaction::async_transaction_end(),
    /// # ]);
    /// # let mut w5500 = W5500::new(spi);
    /// let buf: [u8; 1] = [0];
    /// w5500.set_sn_rx_buf(Sn::Sn0, 0, &buf).await?;
    /// # Ok(()) }
    /// ```
    fn set_sn_rx_buf<'a>(
        &'a mut self,
        sn: Sn,
        ptr: u16,
        buf: &'a [u8],
    ) -> SetSnRxBufFuture<'a, Self> {
        async move { self.write(ptr, sn.rx_block(), buf).await }
    }
}
