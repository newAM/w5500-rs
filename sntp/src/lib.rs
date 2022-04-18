//! SNTP client for the [Wiznet W5500] SPI internet offload chip.
//!
//! # Limitations
//!
//! * No support for message digests
//!
//! # Feature Flags
//!
//! All features are disabled by default.
//!
//! * `embedded-hal`: Passthrough to [`w5500-hl`].
//! * `std`: Passthrough to [`w5500-hl`].
//! * `defmt`: Enable logging with `defmt`. Also a passthrough to [`w5500-hl`].
//! * `log`: Enable logging with `log`.
//! * `chrono`: Enable conversion to `chrono::naive::NaiveDateTime`.
//! * `time`: Enable conversion to `time::PrimitiveDateTime`.
//! * `num-rational`: Enable conversion to `num_rational::Ratio`.
//!
//! # Reference Documentation
//!
//! * [RFC 4330](https://www.rfc-editor.org/rfc/rfc4330.html)
//!
//! [`w5500-hl`]: https://github.com/newAM/w5500-hl-rs
//! [Wiznet W5500]: https://www.wiznet.io/product-item/w5500/
#![cfg_attr(docsrs, feature(doc_cfg), feature(doc_auto_cfg))]
#![cfg_attr(all(not(feature = "std"), not(test)), no_std)]
#![deny(unsafe_code)]
#![warn(missing_docs)]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

mod fixed_point;
mod timestamp;

pub use fixed_point::FixedPoint;
pub use timestamp::Timestamp;
pub use w5500_hl as hl;
pub use w5500_hl::ll;

use hl::{
    io::{Read, Write},
    Common, Error, Udp, UdpReader, UdpWriter,
};
use ll::{
    net::{Ipv4Addr, SocketAddrV4},
    Registers, Sn, SocketInterrupt, SocketInterruptMask,
};

/// IANA SNTP destination port.
#[cfg(target_os = "none")]
const DST_PORT: u16 = 123;
#[cfg(not(target_os = "none"))]
const DST_PORT: u16 = 12345;

// 3-bit version number indicating the current protocol version
const VERSION_NUMBER: u8 = 4;

/// W5500 SNTP client.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Client {
    sn: Sn,
    port: u16,
    server: SocketAddrV4,
}

impl Client {
    /// Create a new NTP client.
    ///
    /// # Arguments
    ///
    /// * `sn` - The socket number to use for DNS queries.
    /// * `port` - SNTP source port, typically 123 is used.
    /// * `server` - The SNTP server IPv4 address.
    ///   Typically this is a DNS server provided by your DHCP client.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_sntp::{
    ///     ll::{net::Ipv4Addr, Sn},
    ///     Client,
    /// };
    ///
    /// const SNTP_SRC_PORT: u16 = 123;
    /// const SNTP_SERVER: Ipv4Addr = Ipv4Addr::new(216, 239, 35, 4);
    ///
    /// let sntp_client: Client = Client::new(Sn::Sn3, SNTP_SRC_PORT, SNTP_SERVER);
    /// ```
    pub fn new(sn: Sn, port: u16, server: Ipv4Addr) -> Self {
        Self {
            sn,
            port,
            server: SocketAddrV4::new(server, DST_PORT),
        }
    }

    /// Set the SNTP server.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_sntp::{
    ///     ll::{net::Ipv4Addr, Sn},
    ///     Client,
    /// };
    ///
    /// const SNTP_SRC_PORT: u16 = 123;
    /// const SNTP_SERVER: Ipv4Addr = Ipv4Addr::new(216, 239, 35, 4);
    /// const LOCAL_SNTP_SERVER: Ipv4Addr = Ipv4Addr::new(10, 0, 42, 42);
    ///
    /// let mut sntp_client: Client = Client::new(Sn::Sn3, SNTP_SRC_PORT, SNTP_SERVER);
    /// assert_eq!(sntp_client.server(), SNTP_SERVER);
    ///
    /// // change server
    /// sntp_client.set_server(LOCAL_SNTP_SERVER);
    /// assert_eq!(sntp_client.server(), LOCAL_SNTP_SERVER);
    /// ```
    #[inline]
    pub fn set_server(&mut self, server: Ipv4Addr) {
        self.server.set_ip(server)
    }

    /// Get the current SNTP server.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_sntp::{
    ///     ll::{net::Ipv4Addr, Sn},
    ///     Client,
    /// };
    ///
    /// const SNTP_SRC_PORT: u16 = 123;
    /// const SNTP_SERVER: Ipv4Addr = Ipv4Addr::new(216, 239, 35, 4);
    ///
    /// let sntp_client: Client = Client::new(Sn::Sn3, SNTP_SRC_PORT, SNTP_SERVER);
    /// assert_eq!(sntp_client.server(), SNTP_SERVER);
    /// ```
    #[inline]
    pub fn server(&self) -> Ipv4Addr {
        *self.server.ip()
    }

    /// Send a request to the SNTP server.
    ///
    /// This will enable the RECV interrupt for the socket, and mask all others.
    ///
    /// At the moment this does not support adding a transmit timestamp.
    ///
    /// The result can be retrieved with [`on_recv_interrupt`] after the next
    /// RECV interrupt.
    ///
    /// # Errors
    ///
    /// This method can only return:
    ///
    /// * [`Error::Other`]
    /// * [`Error::OutOfMemory`]
    ///   * Sending a request requires 48 bytes of memory in the socket buffers.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # let mut w5500 = w5500_regsim::W5500::default();
    /// use w5500_sntp::{
    ///     ll::{net::Ipv4Addr, Sn},
    ///     Client,
    /// };
    ///
    /// const SNTP_SRC_PORT: u16 = 123;
    /// const SNTP_SERVER: Ipv4Addr = Ipv4Addr::new(216, 239, 35, 4);
    ///
    /// let sntp_client: Client = Client::new(Sn::Sn3, SNTP_SRC_PORT, SNTP_SERVER);
    /// sntp_client.request(&mut w5500)?;
    /// # Ok::<(), w5500_hl::Error<std::io::Error>>(())
    /// ```
    ///
    /// [`on_recv_interrupt`]: Self::on_recv_interrupt
    pub fn request<W5500: Registers>(&self, w5500: &mut W5500) -> Result<(), Error<W5500::Error>> {
        const LI: u8 = (LeapIndicator::NoWarning as u8) << 6;
        const VN: u8 = VERSION_NUMBER << 3;
        const MODE: u8 = Mode::Client as u8;

        const STRATUM: u8 = 0;
        const POLL: u8 = 0;
        const PRECISION: u8 = 0;

        // https://www.rfc-editor.org/rfc/rfc4330.html#section-4
        #[rustfmt::skip]
        const REQUEST_PKT: [u8; 48] = [
            LI | VN | MODE, STRATUM, POLL, PRECISION,
            // root delay
            0, 0, 0, 0,
            // root dispersion
            0, 0, 0, 0,
            // reference identifier
            0, 0, 0, 0,
            // reference timestamp
            0, 0, 0, 0, 0, 0, 0, 0,
            // originate timestamp
            0, 0, 0, 0, 0, 0, 0, 0,
            // receive timestamp
            0, 0, 0, 0, 0, 0, 0, 0,
            // transmit timestamp
            // in the future this can be provided as an argument
            0, 0, 0, 0, 0, 0, 0, 0,
        ];

        let simr: u8 = w5500.simr()?;
        w5500.set_simr(self.sn.bitmask() | simr)?;
        const MASK: SocketInterruptMask = SocketInterruptMask::ALL_MASKED.unmask_recv();
        w5500.set_sn_imr(self.sn, MASK)?;
        w5500.close(self.sn)?;
        w5500.udp_bind(self.sn, self.port)?;

        let mut writer: UdpWriter<W5500> = w5500.udp_writer(self.sn)?;
        writer.write_all(&REQUEST_PKT)?;
        writer.udp_send_to(&self.server)?;

        Ok(())
    }

    /// Read a reply from the server.
    ///
    /// You should only call this method after sending a [`request`] and
    /// receiving a RECV interrupt.
    ///
    /// This will clear the pending RECV interrupt.
    ///
    /// # Errors
    ///
    /// This method can only return:
    ///
    /// * [`Error::Other`]
    /// * [`Error::WouldBlock`]
    ///   * In addition to being returned if there is no data to read this can
    ///     also be returned when receiving invalid packets.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # let mut w5500 = w5500_regsim::W5500::default();
    /// use w5500_sntp::{
    ///     hl::Error,
    ///     ll::{net::Ipv4Addr, Sn},
    ///     Client, Reply,
    /// };
    ///
    /// const SNTP_SRC_PORT: u16 = 123;
    /// const SNTP_SERVER: Ipv4Addr = Ipv4Addr::new(216, 239, 35, 4);
    ///
    /// let sntp_client: Client = Client::new(Sn::Sn3, SNTP_SRC_PORT, SNTP_SERVER);
    /// sntp_client.request(&mut w5500)?;
    ///
    /// // ... wait for RECV interrupt with a timeout
    ///
    /// let reply: Reply = match sntp_client.on_recv_interrupt(&mut w5500) {
    ///     Err(Error::WouldBlock) => todo!("implement retry logic here"),
    ///     Err(e) => todo!("handle error: {:?}", e),
    ///     Ok(reply) => reply,
    /// };
    /// # Ok::<(), w5500_hl::Error<std::io::Error>>(())
    /// ```
    ///
    /// [`request`]: Self::request
    pub fn on_recv_interrupt<W5500: Registers>(
        &self,
        w5500: &mut W5500,
    ) -> Result<Reply, Error<W5500::Error>> {
        let sn_ir: SocketInterrupt = w5500.sn_ir(self.sn)?;
        if sn_ir.any_raised() {
            w5500.set_sn_ir(self.sn, sn_ir)?;
        }

        let mut buf: [u8; 48] = [0; 48];
        let mut reader: UdpReader<W5500> = w5500.udp_reader(self.sn)?;

        if reader.header().origin != self.server {
            debug!("unexpected packet from {}", reader.header().origin);
            reader.done()?;
            return Err(Error::WouldBlock);
        }
        reader.read_exact(&mut buf)?;
        reader.done()?;

        match Mode::try_from(buf[0] & 0b111) {
            Ok(Mode::Server) => (),
            Ok(mode) => {
                warn!("invalid mode for reply: {:?}", mode);
                return Err(Error::WouldBlock);
            }
            Err(value) => {
                warn!("invalid value for mode: {}", value);
                return Err(Error::WouldBlock);
            }
        };

        let version_number: u8 = (buf[0] >> 3) & 0b111;
        if version_number != VERSION_NUMBER {
            warn!("unsupported version number: {}", version_number);
            return Err(Error::WouldBlock);
        }

        let leap_indicator: LeapIndicator = LeapIndicator::from_bits(buf[0] >> 6);

        let stratum: Stratum = match buf[1].try_into() {
            Ok(stratum) => stratum,
            Err(value) => {
                warn!("invalid value for stratum: {}", value);
                return Err(Error::WouldBlock);
            }
        };

        let poll: u8 = buf[2];
        if poll != 0 {
            // poll is copied from request for unicast/multicast
            warn!("poll value should be zero not {}", poll);
            return Err(Error::WouldBlock);
        }

        Ok(Reply {
            leap_indicator,
            stratum,
            precision: buf[3] as i8,
            root_delay: FixedPoint {
                bits: u32::from_be_bytes(buf[4..8].try_into().unwrap()),
            },
            root_dispersion: FixedPoint {
                bits: u32::from_be_bytes(buf[8..12].try_into().unwrap()),
            },
            reference_identifier: buf[12..16].try_into().unwrap(),
            reference_timestamp: Timestamp {
                bits: u64::from_be_bytes(buf[16..24].try_into().unwrap()),
            },
            originate_timestamp: Timestamp {
                bits: u64::from_be_bytes(buf[24..32].try_into().unwrap()),
            },
            receive_timestamp: Timestamp {
                bits: u64::from_be_bytes(buf[32..40].try_into().unwrap()),
            },
            transmit_timestamp: Timestamp {
                bits: u64::from_be_bytes(buf[40..48].try_into().unwrap()),
            },
        })
    }
}

/// Reply from the SNTP server.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Reply {
    /// Leap indicator warning of an impending leap second to be
    /// inserted/deleted in the last minute of the current day.
    pub leap_indicator: LeapIndicator,
    /// Stratum.
    pub stratum: Stratum,
    /// This is an eight-bit signed integer used as an exponent of
    /// two, where the resulting value is the precision of the system clock
    /// in seconds.  This field is significant only in server messages, where
    /// the values range from -6 for mains-frequency clocks to -20 for
    /// microsecond clocks found in some workstations.
    pub precision: i8,
    /// Total roundtrip delay to the primary reference source, in seconds.
    ///
    /// The values range from negative values of a few milliseconds to positive
    /// values of several hundred milliseconds.
    pub root_delay: FixedPoint,
    /// The maximum error due to the clock frequency tolerance, in seconds.
    pub root_dispersion: FixedPoint,
    /// For [`Stratum::KoD`] and [`Stratum::Primary`] the value is a
    /// four-character ASCII string, left justified and zero padded to 32 bits.
    ///
    /// For [`Stratum::Secondary`], the value is the 32-bit IPv4 address of
    /// the synchronization source.
    pub reference_identifier: [u8; 4],
    /// This field is the time the system clock was last set or corrected.
    pub reference_timestamp: Timestamp,
    /// This is the time at which the request departed the client for the server.
    pub originate_timestamp: Timestamp,
    /// The time at which the request arrived at the server.
    pub receive_timestamp: Timestamp,
    /// The time at which the reply departed the server.
    pub transmit_timestamp: Timestamp,
}

/// Leap indicator.
///
/// This is a two-bit code warning of an impending leap second to be
/// inserted/deleted in the last minute of the current day.
///
/// # References
///
/// * [RFC 4330 Section 4](https://datatracker.ietf.org/doc/html/rfc4330#section-4)
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum LeapIndicator {
    /// No warning
    NoWarning = 0,
    /// Last minute has 61 seconds
    LastMin61Sec = 1,
    /// Last minute has 59 seconds
    LastMin59Sec = 2,
    /// Alarm condition (clock not synchronized)
    Alarm = 3,
}

impl LeapIndicator {
    pub(crate) fn from_bits(bits: u8) -> Self {
        match bits & 0b11 {
            x if x == (Self::NoWarning as u8) => Self::NoWarning,
            x if x == (Self::LastMin61Sec as u8) => Self::LastMin61Sec,
            x if x == (Self::LastMin59Sec as u8) => Self::LastMin59Sec,
            x if x == (Self::Alarm as u8) => Self::Alarm,
            _ => unreachable!(),
        }
    }
}

/// SNTP modes.
///
/// # References
///
/// * [RFC 4330 Section 4](https://datatracker.ietf.org/doc/html/rfc4330#section-4)
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
#[non_exhaustive]
enum Mode {
    SymmetricActive = 1,
    SymmetricPassive = 2,
    Client = 3,
    Server = 4,
    Broadcast = 5,
}

impl TryFrom<u8> for Mode {
    type Error = u8;

    fn try_from(bits: u8) -> Result<Self, Self::Error> {
        match bits {
            x if x == (Self::SymmetricActive as u8) => Ok(Self::SymmetricActive),
            x if x == (Self::SymmetricPassive as u8) => Ok(Self::SymmetricPassive),
            x if x == (Self::Client as u8) => Ok(Self::Client),
            x if x == (Self::Server as u8) => Ok(Self::Server),
            x if x == (Self::Broadcast as u8) => Ok(Self::Broadcast),
            x => Err(x),
        }
    }
}

/// Stratum, device's distance to the reference clock.
#[non_exhaustive]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Stratum {
    /// Kiss-o'-Death
    ///
    /// See [RFC 4330 Section 8] for information about KoD.
    ///
    /// [RFC 4330 Section 8]: https://www.rfc-editor.org/rfc/rfc4330.html#section-8
    KoD,
    /// Primary reference (e.g., synchronized by radio clock)
    Primary,
    /// Secondary reference (synchronized by NTP or SNTP)
    Secondary(u8),
}

impl TryFrom<u8> for Stratum {
    type Error = u8;

    fn try_from(bits: u8) -> Result<Self, Self::Error> {
        match bits {
            0 => Ok(Self::KoD),
            1 => Ok(Self::Primary),
            2..=15 => Ok(Self::Secondary(bits)),
            _ => Err(bits),
        }
    }
}
