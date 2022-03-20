//! Platform agnostic rust driver for the [Wiznet W5500] internet offload chip.
//!
//! This crate contains higher level (hl) socket operations, built on-top of my
//! other crate, [w5500-ll], which contains register accessors, and networking
//! data types for the W5500.
//!
//! # Design
//!
//! There are no separate socket structures.
//! The [`Tcp`] and [`Udp`] traits provided in this crate simply extend the
//! [`Registers`] trait provided in [w5500-ll].
//! This makes for a less ergonomic API, but a much more portable API because
//! there are no mutexes or runtime checks to enable socket structures to share
//! ownership of the underlying W5500 device.
//!
//! You will likely want to wrap up the underlying structure that implements
//! the [`Registers`], [`Tcp`], and [`Udp`] traits to provide separate socket
//! structures utilizing whatever Mutex is available for your platform / RTOS.
//!
//! # Feature Flags
//!
//! All features are disabled by default.
//!
//! * `defmt`: Passthrough to [w5500-ll].
//! * `embedded-hal`: Passthrough to [w5500-ll].
//! * `std`: Passthrough to [w5500-ll].
//!
//! # Examples
//!
//! UDP sockets
//!
//! ```no_run
//! # use embedded_hal_mock as h;
//! # let mut w5500 = w5500_ll::blocking::vdm::W5500::new(h::spi::Mock::new(&[]), h::pin::Mock::new(&[]));
//! use w5500_hl::ll::{
//!     net::{Ipv4Addr, SocketAddrV4},
//!     Registers,
//!     Sn::Sn0,
//! };
//! use w5500_hl::Udp;
//!
//! // open Sn0 as a UDP socket on port 1234
//! w5500.udp_bind(Sn0, 1234)?;
//!
//! // send 4 bytes to 192.168.2.4:8080, and get the number of bytes transmitted
//! let data: [u8; 4] = [0, 1, 2, 3];
//! let destination = SocketAddrV4::new(Ipv4Addr::new(192, 168, 2, 4), 8080);
//! let tx_bytes = w5500.udp_send_to(Sn0, &data, &destination)?;
//! # Ok::<(), w5500_hl::ll::blocking::vdm::Error<_, _>>(())
//! ```
//!
//! TCP streams (client)
//!
//! ```no_run
//! # use embedded_hal_mock as h;
//! # let mut w5500 = w5500_ll::blocking::vdm::W5500::new(h::spi::Mock::new(&[]), h::pin::Mock::new(&[]));
//! use w5500_hl::ll::{
//!     net::{Ipv4Addr, SocketAddrV4},
//!     Registers, Sn,
//! };
//! use w5500_hl::Tcp;
//!
//! const MQTT_SOCKET: Sn = Sn::Sn0;
//! const MQTT_SOURCE_PORT: u16 = 33650;
//! const MQTT_SERVER: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(192, 168, 2, 10), 1883);
//!
//! // initiate a TCP connection to a MQTT server
//! w5500.tcp_connect(MQTT_SOCKET, MQTT_SOURCE_PORT, &MQTT_SERVER)?;
//! # Ok::<(), w5500_hl::ll::blocking::vdm::Error<_, _>>(())
//! ```
//!
//! TCP listeners (server)
//!
//! ```no_run
//! # use embedded_hal_mock as h;
//! # let mut w5500 = w5500_ll::blocking::vdm::W5500::new(h::spi::Mock::new(&[]), h::pin::Mock::new(&[]));
//! use w5500_hl::ll::{
//!     net::{Ipv4Addr, SocketAddrV4},
//!     Registers, Sn,
//! };
//! use w5500_hl::Tcp;
//!
//! const HTTP_SOCKET: Sn = Sn::Sn1;
//! const HTTP_PORT: u16 = 80;
//!
//! // serve HTTP
//! w5500.tcp_listen(HTTP_SOCKET, HTTP_PORT)?;
//! # Ok::<(), w5500_hl::ll::blocking::vdm::Error<_, _>>(())
//! ```
//!
//! [`Registers`]: https://docs.rs/w5500-ll/latest/w5500_ll/trait.Registers.html
//! [`std::net`]: https://doc.rust-lang.org/std/net/index.html
//! [`Tcp`]: https://docs.rs/w5500-hl/0.7.1/w5500_hl/trait.Tcp.html
//! [`Udp`]: https://docs.rs/w5500-hl/0.7.1/w5500_hl/trait.Udp.html
//! [w5500-ll]: https://github.com/newAM/w5500-ll-rs
//! [Wiznet W5500]: https://www.wiznet.io/product-item/w5500/
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(all(not(feature = "std"), not(test)), no_std)]

mod tcp;
mod udp;

#[cfg(feature = "defmt")]
use dfmt as defmt;

use core::cmp::min;
use ll::{Registers, Sn, SocketCommand, SocketStatus, SOCKETS};

pub use tcp::{Tcp, TcpReader};
pub use udp::{Udp, UdpHeader, UdpReader};
pub use w5500_ll as ll;

/// Networking data types.
///
/// These are exported from [`w5500_ll::net`].
pub mod net {
    pub use w5500_ll::net::{Eui48Addr, Ipv4Addr, SocketAddrV4};
}

use net::{Ipv4Addr, SocketAddrV4};

fn port_is_unique<T: ?Sized, E>(w5500: &mut T, socket: Sn, port: u16) -> Result<bool, E>
where
    T: Registers<Error = E>,
{
    const CLOSED_STATUS: [Result<SocketStatus, u8>; 3] = [
        Ok(SocketStatus::Closed),
        Ok(SocketStatus::CloseWait),
        Ok(SocketStatus::Closing),
    ];
    for socket in SOCKETS.iter().filter(|s| s != &&socket) {
        if w5500.sn_port(*socket)? == port {
            let status = w5500.sn_sr(*socket)?;
            if !CLOSED_STATUS.iter().any(|x| x == &status) {
                return Ok(false);
            }
        }
    }
    Ok(true)
}

/// Higher level W5500 errors.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error<E> {
    /// Unexpected "end of file".
    ///
    /// Returned when an operation could only succeed if it read a particular
    /// number of bytes but only a smaller number of bytes could be read; for
    /// example this may occur when a UDP packet is truncated.
    UnexpectedEof,
    /// A write operation ran out of memory in the socket buffer.
    OutOfMemory,
    /// The operation needs to block to complete, but the blocking operation was
    /// requested to not occur.
    ///
    /// This is the same concept as the [`nb`] crate, but localized to prevent
    /// needless abstraction.
    ///
    /// [`nb`]: (https://docs.rs/nb/latest/nb/index.html)
    WouldBlock,
    /// Errors from the [`Registers`] trait implementation.
    Other(E),
}

impl<E> From<E> for Error<E> {
    fn from(error: E) -> Error<E> {
        Error::Other(error)
    }
}

/// Turns a non-blocking W5500 expression `$e` into a blocking operation.
///
/// This is accomplished by continuously calling the expression `$e` until it no
/// longer returns [`Error::WouldBlock`].
///
/// # Input
///
/// An expression `$e` that evaluates to `Result<T, Error<E>>`
///
/// # Output
///
/// - `Ok(t)` if `$e` evaluates to `Ok(t)`
/// - `Err(e)` if `$e` evaluates to any error that is not `Err(Error::WouldBlock)`
#[macro_export]
macro_rules! block {
    ($e:expr) => {
        loop {
            #[allow(unreachable_patterns)]
            match $e {
                Err($crate::Error::WouldBlock) => {}
                Err(e) => break Err(e),
                Ok(x) => break Ok(x),
            }
        }
    };
}

/// Enumeration of all possible methods to seek the W5500 socket buffers.
///
/// This is designed to be similar to [`std::io::SeekFrom`].
///
/// [`std::io::SeekFrom`]: https://doc.rust-lang.org/std/io/enum.SeekFrom.html
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SeekFrom {
    /// Sets the offset to the provided number of bytes.
    Start(u16),
    /// Sets the offset to the end plus the specified number of bytes.
    End(i16),
    /// Sets the offset to the current position plus the specified number of bytes.
    Current(i16),
}

// TODO: use wrapping_add_signed when stabilized
// https://github.com/rust-lang/rust/issues/87840
// https://github.com/rust-lang/rust/blob/21b0325c68421b00c6c91055ac330bd5ffe1ea6b/library/core/src/num/uint_macros.rs#L1205
fn wrapping_add_signed(ptr: u16, offset: i16) -> u16 {
    ptr.wrapping_add(offset as u16)
}

impl SeekFrom {
    #[must_use]
    #[inline]
    fn new_ptr(self, ptr: u16, head: u16, tail: u16) -> u16 {
        match self {
            SeekFrom::Start(offset) => head.wrapping_add(offset),
            SeekFrom::End(offset) => wrapping_add_signed(tail, offset),
            SeekFrom::Current(offset) => wrapping_add_signed(ptr, offset),
        }
    }
}

/// The `Seek` trait provides a cursor which can be moved within a stream of
/// bytes.
///
/// This is used for navigating the socket buffers, and it is designed to be
/// similar to [`std::io::Seek`].
///
/// [`std::io::Seek`]: https://doc.rust-lang.org/stable/std/io/trait.Seek.html
pub trait Seek {
    /// Seek to an offset, in bytes, within the socket buffer.
    ///
    /// Seeking beyond the limits will result in wrapping around to the next
    /// valid address.
    ///
    /// # Limits
    ///
    /// * [`Writer`] is limited by socket free size.
    /// * [`UdpReader`] is limited by the received size or the UDP datagram length,
    ///   whichever is less.
    /// * [`TcpReader`] is limited by the received size.
    fn seek(&mut self, pos: SeekFrom);

    /// Rewind to the beginning of the stream.
    ///
    /// This is a convenience method, equivalent to `seek(SeekFrom::Start(0))`.
    fn rewind(&mut self) {
        self.seek(SeekFrom::Start(0))
    }

    /// Return the length of the stream, in bytes.
    ///
    /// * For [`Writer`] this returns the socket free size.
    /// * For [`TcpReader`] this returns the received size.
    /// * For [`UdpReader`] this returns the received size or the UDP datagram
    ///   length, whichever is less.
    fn stream_len(&self) -> u16;

    /// Returns the current seek position from the start of the stream.
    fn stream_position(&self) -> u16;

    /// Remaining bytes in the socket buffer from the current seek position.
    fn remain(&self) -> u16;
}

/// Socket reader trait.
///
/// This is implemented by [`TcpReader`] and [`UdpReader`].
pub trait Read<'a, W: Registers> {
    /// Read data from the UDP socket, and return the number of bytes read.
    fn read(&mut self, buf: &mut [u8]) -> Result<u16, W::Error>;

    /// Read the exact number of bytes required to fill `buf`.
    ///
    /// This function reads as many bytes as necessary to completely fill the
    /// specified buffer `buf`.
    ///
    /// # Errors
    ///
    /// This method can only return:
    ///
    /// * [`Error::Other`]
    /// * [`Error::UnexpectedEof`]
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Error<W::Error>>;

    /// Mark the data as read, removing the data from the queue.
    ///
    /// For a UDP reader this removes the UDP datagram from the queue.
    fn done(self) -> Result<&'a mut W, W::Error>;

    /// All data and return it to the queue.
    ///
    /// For a UDP reader this returns the UDP datagram to the queue.
    fn ignore(self) -> &'a mut W;
}

/// Streaming writer for a TCP or UDP socket buffer.
///
/// Created with [`Common::writer`].
///
/// # Example
///
/// ```no_run
/// # use embedded_hal_mock as h;
/// # let mut w5500 = w5500_ll::blocking::vdm::W5500::new(h::spi::Mock::new(&[]), h::pin::Mock::new(&[]));
/// use w5500_hl::{
///     ll::{Registers, Sn::Sn0},
///     net::{Ipv4Addr, SocketAddrV4},
///     Udp,
///     Common,
///     Writer,
/// };
///
/// const DEST: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(192, 0, 2, 1), 8081);
///
/// w5500.udp_bind(Sn0, 8080)?;
///
/// let mut udp_writer: Writer<_> = w5500.writer(Sn0)?;
///
/// let data_header: [u8; 10] = [0; 10];
/// let n_written: u16 = udp_writer.write(&data_header)?;
/// assert_eq!(usize::from(n_written), data_header.len());
///
/// let data: [u8; 123] = [0; 123];
/// let n_written: u16 = udp_writer.write(&data)?;
/// assert_eq!(usize::from(n_written), data.len());
///
/// udp_writer.udp_send_to(&DEST)?;
/// # Ok::<(), w5500_hl::ll::blocking::vdm::Error<_, _>>(())
/// ```
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Writer<'a, W: Registers> {
    w5500: &'a mut W,
    sn: Sn,
    head_ptr: u16,
    tail_ptr: u16,
    ptr: u16,
}

impl<'a, W: Registers> Seek for Writer<'a, W> {
    fn seek(&mut self, pos: SeekFrom) {
        self.ptr = pos.new_ptr(self.ptr, self.head_ptr, self.tail_ptr);
    }

    fn stream_len(&self) -> u16 {
        self.tail_ptr.wrapping_sub(self.head_ptr)
    }

    fn stream_position(&self) -> u16 {
        self.ptr.wrapping_sub(self.head_ptr)
    }

    fn remain(&self) -> u16 {
        self.tail_ptr.wrapping_sub(self.ptr)
    }
}

impl<'a, W: Registers> Writer<'a, W> {
    /// Write data to the socket buffer, and return the number of bytes written.
    pub fn write(&mut self, buf: &[u8]) -> Result<u16, W::Error> {
        let write_size: u16 = min(self.remain(), buf.len().try_into().unwrap_or(u16::MAX));
        if write_size != 0 {
            self.w5500
                .set_sn_tx_buf(self.sn, self.ptr, &buf[..usize::from(write_size)])?;
            self.ptr = self.ptr.wrapping_add(write_size);

            Ok(write_size)
        } else {
            Ok(0)
        }
    }

    /// Writes all the data, returning [`Error::OutOfMemory`] if the size of
    /// `buf` exceeds the free memory available in the socket buffer.
    ///
    /// # Errors
    ///
    /// This method can only return:
    ///
    /// * [`Error::Other`]
    /// * [`Error::OutOfMemory`]
    pub fn write_all(&mut self, buf: &[u8]) -> Result<(), Error<W::Error>> {
        let buf_len: u16 = buf.len().try_into().unwrap_or(u16::MAX);
        let write_size: u16 = min(self.remain(), buf_len);
        if write_size != buf_len {
            Err(Error::OutOfMemory)
        } else {
            self.w5500.set_sn_tx_buf(self.sn, self.ptr, buf)?;
            self.ptr = self.ptr.wrapping_add(write_size);
            Ok(())
        }
    }

    /// Send all data previously written with [`write`] and [`write_all`].
    ///
    /// For UDP sockets the destination is set by the last call to
    /// [`Registers::set_sn_dest`], [`Udp::udp_send_to`], or
    /// [`Writer::udp_send_to`].
    ///
    /// [`write`]: Writer::write
    /// [`write_all`]: Writer::write_all
    pub fn send(self) -> Result<&'a mut W, W::Error> {
        self.w5500.set_sn_tx_wr(self.sn, self.ptr)?;
        self.w5500.set_sn_cr(self.sn, SocketCommand::Send)?;
        Ok(self.w5500)
    }

    /// Send all data previously written with [`Writer::write`] and
    /// [`Writer::write_all`] to the given address.
    ///
    /// # Panics
    ///
    /// * (debug) The socket must be opened as a UDP socket.
    pub fn udp_send_to(self, addr: &SocketAddrV4) -> Result<&'a mut W, W::Error> {
        debug_assert_eq!(self.w5500.sn_sr(self.sn)?, Ok(SocketStatus::Udp));

        self.w5500.set_sn_dest(self.sn, addr)?;
        self.send()
    }
}

/// Methods common to all W5500 socket types.
pub trait Common: Registers {
    /// Returns the socket address.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use embedded_hal_mock as h;
    /// # let mut w5500 = w5500_ll::blocking::vdm::W5500::new(h::spi::Mock::new(&[]), h::pin::Mock::new(&[]));
    /// use w5500_hl::ll::{Registers, Sn::Sn0};
    /// use w5500_hl::{Common, Udp};
    ///
    /// w5500.udp_bind(Sn0, 8080)?;
    /// let local_addr = w5500.local_addr(Sn0)?;
    /// # Ok::<(), w5500_hl::ll::blocking::vdm::Error<_, _>>(())
    /// ```
    fn local_addr(&mut self, sn: Sn) -> Result<SocketAddrV4, Self::Error> {
        let ip: Ipv4Addr = self.sipr()?;
        let port: u16 = self.sn_port(sn)?;
        Ok(SocketAddrV4::new(ip, port))
    }

    /// Close a socket.
    ///
    /// This will not poll for completion, the socket may not be closed after
    /// this method has returned.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use embedded_hal_mock as h;
    /// # let mut w5500 = w5500_ll::blocking::vdm::W5500::new(h::spi::Mock::new(&[]), h::pin::Mock::new(&[]));
    /// use w5500_hl::ll::{Registers, Sn::Sn0};
    /// use w5500_hl::Common;
    ///
    /// w5500.close(Sn0)?;
    /// # Ok::<(), w5500_hl::ll::blocking::vdm::Error<_, _>>(())
    /// ```
    fn close(&mut self, sn: Sn) -> Result<(), Self::Error> {
        self.set_sn_cr(sn, SocketCommand::Close)
    }

    /// Returns `true` if the socket state is [Closed].
    ///
    /// **Note:** This does not include states that indicate the socket is about
    /// to close, such as [Closing].
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use embedded_hal_mock as h;
    /// # let mut w5500 = w5500_ll::blocking::vdm::W5500::new(h::spi::Mock::new(&[]), h::pin::Mock::new(&[]));
    /// use w5500_hl::ll::{Registers, Sn::Sn0};
    /// use w5500_hl::{Common, Udp};
    ///
    /// w5500.close(Sn0)?;
    /// assert!(w5500.is_state_closed(Sn0)?);
    /// w5500.udp_bind(Sn0, 8080)?;
    /// assert!(!w5500.is_state_closed(Sn0)?);
    /// # Ok::<(), w5500_hl::ll::blocking::vdm::Error<_, _>>(())
    /// ```
    ///
    /// [Closed]: w5500_ll::SocketStatus::Closed
    /// [Closing]: w5500_ll::SocketStatus::Closing
    fn is_state_closed(&mut self, sn: Sn) -> Result<bool, Self::Error> {
        Ok(self.sn_sr(sn)? == Ok(SocketStatus::Closed))
    }

    /// Returns `true` if the socket state is any valid TCP state as described
    /// in [RFC 793].
    ///
    /// Valid TCP states include:
    ///
    /// * [Closed]
    /// * [Listen]
    /// * [SynSent]
    /// * [SynRecv]
    /// * [Established]
    /// * [FinWait]
    /// * [Closing]
    /// * [CloseWait]
    /// * [TimeWait]
    /// * [LastAck]
    ///
    /// **Note:** This **does not** include the W5500 [Init] state.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use embedded_hal_mock as h;
    /// # let mut w5500 = w5500_ll::blocking::vdm::W5500::new(h::spi::Mock::new(&[]), h::pin::Mock::new(&[]));
    /// use w5500_hl::ll::{Registers, Sn::Sn0};
    /// use w5500_hl::{Common, Udp};
    ///
    /// w5500.close(Sn0)?;
    /// assert!(w5500.is_state_tcp(Sn0)?);
    /// w5500.udp_bind(Sn0, 8080)?;
    /// assert!(!w5500.is_state_tcp(Sn0)?);
    /// # Ok::<(), w5500_hl::ll::blocking::vdm::Error<_, _>>(())
    /// ```
    ///
    /// [RFC 793]: https://tools.ietf.org/html/rfc793
    /// [Init]: w5500_ll::SocketStatus::Init
    /// [Closed]: w5500_ll::SocketStatus::Closed
    /// [Listen]: w5500_ll::SocketStatus::Listen
    /// [SynSent]: w5500_ll::SocketStatus::SynSent
    /// [SynRecv]: w5500_ll::SocketStatus::SynRecv
    /// [Established]: w5500_ll::SocketStatus::Established
    /// [FinWait]: w5500_ll::SocketStatus::FinWait
    /// [Closing]: w5500_ll::SocketStatus::Closing
    /// [CloseWait]: w5500_ll::SocketStatus::CloseWait
    /// [TimeWait]: w5500_ll::SocketStatus::TimeWait
    /// [LastAck]: w5500_ll::SocketStatus::LastAck
    fn is_state_tcp(&mut self, sn: Sn) -> Result<bool, Self::Error> {
        // Hopefully the compiler will optimize this to check that the state is
        // not MACRAW, UDP, or INIT.
        // Leaving it as-is since the code is more readable this way.
        Ok(matches!(
            self.sn_sr(sn)?,
            Ok(SocketStatus::Closed)
                | Ok(SocketStatus::Listen)
                | Ok(SocketStatus::SynSent)
                | Ok(SocketStatus::SynRecv)
                | Ok(SocketStatus::Established)
                | Ok(SocketStatus::FinWait)
                | Ok(SocketStatus::Closing)
                | Ok(SocketStatus::CloseWait)
                | Ok(SocketStatus::TimeWait)
                | Ok(SocketStatus::LastAck)
        ))
    }

    /// Returns `true` if the socket state is [Udp].
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use embedded_hal_mock as h;
    /// # let mut w5500 = w5500_ll::blocking::vdm::W5500::new(h::spi::Mock::new(&[]), h::pin::Mock::new(&[]));
    /// use w5500_hl::ll::{Registers, Sn::Sn0};
    /// use w5500_hl::{Common, Udp};
    ///
    /// w5500.close(Sn0)?;
    /// assert!(!w5500.is_state_udp(Sn0)?);
    /// w5500.udp_bind(Sn0, 8080)?;
    /// assert!(w5500.is_state_udp(Sn0)?);
    /// # Ok::<(), w5500_hl::ll::blocking::vdm::Error<_, _>>(())
    /// ```
    ///
    /// [Udp]: w5500_ll::SocketStatus::Udp
    fn is_state_udp(&mut self, sn: Sn) -> Result<bool, Self::Error> {
        Ok(self.sn_sr(sn)? == Ok(SocketStatus::Udp))
    }

    /// Create a socket writer.
    ///
    /// This returns a [`Writer`] structure, which contains functions to
    /// stream data into the W5500 socket buffers incrementally.
    ///
    /// This is useful for writing large packets that are too large to stage
    /// in the memory of your microcontroller.
    ///
    /// The socket should be opened as a TCP / UDP socket before calling this
    /// method.
    ///
    /// # Example
    ///
    /// ```no_std
    /// # use embedded_hal_mock as h;
    /// # let mut w5500 = w5500_ll::blocking::vdm::W5500::new(h::spi::Mock::new(&[]), h::pin::Mock::new(&[]));
    /// use w5500_hl::{
    ///     ll::{Registers, Sn::Sn0},
    ///     net::{Ipv4Addr, SocketAddrV4},
    ///     Udp,
    ///     Writer,
    /// };
    ///
    /// const DEST: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(192, 0, 2, 1), 8081);
    ///
    /// w5500.udp_bind(Sn0, 8080)?;
    ///
    /// let mut udp_writer: Writer<_> = w5500.writer(Sn0)?;
    /// ```
    fn writer(&mut self, sn: Sn) -> Result<Writer<Self>, Self::Error>
    where
        Self: Sized,
    {
        let sn_tx_fsr: u16 = self.sn_tx_fsr(sn)?;
        let sn_tx_wr: u16 = self.sn_tx_wr(sn)?;

        Ok(Writer {
            w5500: self,
            sn,
            head_ptr: sn_tx_wr,
            tail_ptr: sn_tx_wr.wrapping_add(sn_tx_fsr),
            ptr: sn_tx_wr,
        })
    }
}

/// Implement the common socket trait for any structure that implements [`w5500_ll::Registers`].
impl<T> Common for T where T: Registers {}

#[cfg(test)]
mod tests {
    use core::convert::Infallible;

    use super::*;

    struct MockRegisters {
        pub socket_ports: [u16; SOCKETS.len()],
        pub socket_status: [SocketStatus; SOCKETS.len()],
    }

    impl Registers for MockRegisters {
        type Error = Infallible;

        fn read(&mut self, _address: u16, _block: u8, _data: &mut [u8]) -> Result<(), Self::Error> {
            unimplemented!()
        }

        fn write(&mut self, _address: u16, _block: u8, _data: &[u8]) -> Result<(), Self::Error> {
            unimplemented!()
        }

        fn sn_port(&mut self, socket: Sn) -> Result<u16, Self::Error> {
            Ok(self.socket_ports[usize::from(socket)])
        }

        fn sn_sr(&mut self, socket: Sn) -> Result<Result<SocketStatus, u8>, Self::Error> {
            Ok(Ok(self.socket_status[usize::from(socket)]))
        }
    }

    #[test]
    fn test_port_is_unique() {
        let mut mock = MockRegisters {
            socket_ports: [0; SOCKETS.len()],
            socket_status: [SocketStatus::Closed; SOCKETS.len()],
        };
        // basics
        assert!(port_is_unique(&mut mock, Sn::Sn0, 0).unwrap());
        assert!(port_is_unique(&mut mock, Sn::Sn0, 1).unwrap());
        assert!(port_is_unique(&mut mock, Sn::Sn0, u16::MAX).unwrap());

        // do not check our own socket
        mock.socket_status[0] = SocketStatus::Init;
        assert!(port_is_unique(&mut mock, Sn::Sn0, 0).unwrap());

        // other socket on other port
        assert!(port_is_unique(&mut mock, Sn::Sn0, 1).unwrap());

        // other socket on same port
        assert!(!port_is_unique(&mut mock, Sn::Sn1, 0).unwrap());
    }
}
