//! Platform agnostic rust driver for the [Wiznet W5500] internet offload chip.
//!
//! This crate contains higher level (hl) socket operations, built on-top of my
//! other crate, [`w5500-ll`], which contains register accessors, and networking
//! data types for the W5500.
//!
//! # Design
//!
//! There are no separate socket structures.
//! The [`Tcp`] and [`Udp`] traits provided in this crate simply extend the
//! [`Registers`] trait provided in [`w5500-ll`].
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
//! * `defmt`: Passthrough to [`w5500-ll`].
//! * `eh0`: Passthrough to [`w5500-ll`].
//! * `eh1`: Passthrough to [`w5500-ll`].
//!
//! # Examples
//!
//! UDP sockets
//!
//! ```no_run
//! # use ehm::eh1 as h;
//! # let mut w5500 = w5500_ll::eh1::vdm::W5500::new(h::spi::Mock::new(&[]));
//! use w5500_hl::Udp;
//! use w5500_hl::ll::{
//!     Registers,
//!     Sn::Sn0,
//!     net::{Ipv4Addr, SocketAddrV4},
//! };
//!
//! // open Sn0 as a UDP socket on port 1234
//! w5500.udp_bind(Sn0, 1234)?;
//!
//! // send 4 bytes to 192.168.2.4:8080, and get the number of bytes transmitted
//! let data: [u8; 4] = [0, 1, 2, 3];
//! let destination = SocketAddrV4::new(Ipv4Addr::new(192, 168, 2, 4), 8080);
//! let tx_bytes = w5500.udp_send_to(Sn0, &data, &destination)?;
//! # Ok::<(), embedded_hal::spi::ErrorKind>(())
//! ```
//!
//! TCP streams (client)
//!
//! ```no_run
//! # use ehm::eh1 as h;
//! # let mut w5500 = w5500_ll::eh1::vdm::W5500::new(h::spi::Mock::new(&[]));
//! use w5500_hl::Tcp;
//! use w5500_hl::ll::{
//!     Registers, Sn,
//!     net::{Ipv4Addr, SocketAddrV4},
//! };
//!
//! const MQTT_SOCKET: Sn = Sn::Sn0;
//! const MQTT_SOURCE_PORT: u16 = 33650;
//! const MQTT_SERVER: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(192, 168, 2, 10), 1883);
//!
//! // initiate a TCP connection to a MQTT server
//! w5500.tcp_connect(MQTT_SOCKET, MQTT_SOURCE_PORT, &MQTT_SERVER)?;
//! # Ok::<(), embedded_hal::spi::ErrorKind>(())
//! ```
//!
//! TCP listeners (server)
//!
//! ```no_run
//! # use ehm::eh1 as h;
//! # let mut w5500 = w5500_ll::eh1::vdm::W5500::new(h::spi::Mock::new(&[]));
//! use w5500_hl::Tcp;
//! use w5500_hl::ll::{
//!     Registers, Sn,
//!     net::{Ipv4Addr, SocketAddrV4},
//! };
//!
//! const HTTP_SOCKET: Sn = Sn::Sn1;
//! const HTTP_PORT: u16 = 80;
//!
//! // serve HTTP
//! w5500.tcp_listen(HTTP_SOCKET, HTTP_PORT)?;
//! # Ok::<(), embedded_hal::spi::ErrorKind>(())
//! ```
//!
//! [`Registers`]: https://docs.rs/w5500-ll/latest/w5500_ll/trait.Registers.html
//! [`Tcp`]: https://docs.rs/w5500-hl/latest/w5500_hl/trait.Tcp.html
//! [`Udp`]: https://docs.rs/w5500-hl/latest/w5500_hl/trait.Udp.html
//! [`w5500-ll`]: https://crates.io/crates/w5500-ll
//! [Wiznet W5500]: https://docs.wiznet.io/Product/iEthernet/W5500/overview
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(not(test), no_std)]

mod hostname;
pub mod io;
mod tcp;
mod udp;

pub use hostname::{Hostname, TryFromStrError};
pub use ll::net;
use ll::{Registers, SOCKETS, Sn, SocketCommand, SocketStatus};
pub use tcp::{Tcp, TcpReader, TcpWriter};
pub use udp::{Udp, UdpHeader, UdpReader, UdpWriter};
pub use w5500_ll as ll;

use net::{Ipv4Addr, SocketAddrV4};

fn port_is_unique<T, E>(w5500: &mut T, socket: Sn, port: u16) -> Result<bool, E>
where
    T: Registers<Error = E> + ?Sized,
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

/// Methods common to all W5500 socket types.
pub trait Common: Registers {
    /// Returns the socket address.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # let mut w5500 = w5500_ll::eh1::vdm::W5500::new(ehm::eh1::spi::Mock::new(&[]));
    /// use w5500_hl::ll::{Registers, Sn::Sn0};
    /// use w5500_hl::{Common, Udp};
    ///
    /// w5500.udp_bind(Sn0, 8080)?;
    /// let local_addr = w5500.local_addr(Sn0)?;
    /// # Ok::<(), embedded_hal::spi::ErrorKind>(())
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
    /// # let mut w5500 = w5500_ll::eh1::vdm::W5500::new(ehm::eh1::spi::Mock::new(&[]));
    /// use w5500_hl::Common;
    /// use w5500_hl::ll::{Registers, Sn::Sn0};
    ///
    /// w5500.close(Sn0)?;
    /// # Ok::<(), embedded_hal::spi::ErrorKind>(())
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
    /// # let mut w5500 = w5500_ll::eh1::vdm::W5500::new(ehm::eh1::spi::Mock::new(&[]));
    /// use w5500_hl::ll::{Registers, Sn::Sn0};
    /// use w5500_hl::{Common, Udp};
    ///
    /// w5500.close(Sn0)?;
    /// assert!(w5500.is_state_closed(Sn0)?);
    /// w5500.udp_bind(Sn0, 8080)?;
    /// assert!(!w5500.is_state_closed(Sn0)?);
    /// # Ok::<(), embedded_hal::spi::ErrorKind>(())
    /// ```
    ///
    /// [Closed]: w5500_ll::SocketStatus::Closed
    /// [Closing]: w5500_ll::SocketStatus::Closing
    #[allow(clippy::wrong_self_convention)]
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
    /// # let mut w5500 = w5500_ll::eh1::vdm::W5500::new(ehm::eh1::spi::Mock::new(&[]));
    /// use w5500_hl::ll::{Registers, Sn::Sn0};
    /// use w5500_hl::{Common, Udp};
    ///
    /// w5500.close(Sn0)?;
    /// assert!(w5500.is_state_tcp(Sn0)?);
    /// w5500.udp_bind(Sn0, 8080)?;
    /// assert!(!w5500.is_state_tcp(Sn0)?);
    /// # Ok::<(), embedded_hal::spi::ErrorKind>(())
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
    #[allow(clippy::wrong_self_convention)]
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
    /// # let mut w5500 = w5500_ll::eh1::vdm::W5500::new(ehm::eh1::spi::Mock::new(&[]));
    /// use w5500_hl::ll::{Registers, Sn::Sn0};
    /// use w5500_hl::{Common, Udp};
    ///
    /// w5500.close(Sn0)?;
    /// assert!(!w5500.is_state_udp(Sn0)?);
    /// w5500.udp_bind(Sn0, 8080)?;
    /// assert!(w5500.is_state_udp(Sn0)?);
    /// # Ok::<(), embedded_hal::spi::ErrorKind>(())
    /// ```
    ///
    /// [Udp]: w5500_ll::SocketStatus::Udp
    #[allow(clippy::wrong_self_convention)]
    fn is_state_udp(&mut self, sn: Sn) -> Result<bool, Self::Error> {
        Ok(self.sn_sr(sn)? == Ok(SocketStatus::Udp))
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
