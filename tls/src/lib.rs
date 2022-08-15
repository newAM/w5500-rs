//! TLS v1.3 client for the [Wiznet W5500] SPI internet offload chip.
//!
//! This requires roughly 19k of flash for a `thumbv7em-none-eabi` target
//! with `-O3`, debug assertions enabled, and the `p256-cm4` feature.
//! Enabling all logging requires an additional ~40k of flash.
//!
//! # Warning
//!
//! ⚠️ This is in an early alpha state ⚠️
//!
//! All the usual security disclaimers apply here, read the license, your hamster
//! may explode if you use this, don't use this code in production, etc.
//!
//! Additionally this is not secure from side channel attacks.
//!
//! * Encryption may occur in-place in the socket buffers, anything with access
//!   to the physical SPI bus or the SPI device registers can easily intercept
//!   data.
//! * To facilitate the ill-advised encryption in-place in the socket buffers
//!   there is a hacky AES implementation that has little thought put towards
//!   constant-time evaluation.
//!
//! # Limitations
//!
//! At the moment this only supports pre-shared keys.
//! This will not work for majority of web (HTTPS) applications.
//!
//! * Requires a local buffer equal to the socket buffer size.
//!   * TLS record fragmentation makes implementing socket buffer streaming
//!     impractical.
//! * Limited cryptography support
//!   * Cipher: `TLS_AES_128_GCM_SHA256`
//!   * Key Exchange: `secp256r1`
//! * Does not support certificate validation
//! * Does not support client certificates (mutual TLS)
//! * Does not support early data
//! * Does not support serving TLS
//!
//! # Feature Flags
//!
//! All features are disabled by default.
//!
//! * `eh0`: Passthrough to [`w5500-hl`].
//! * `eh1`: Passthrough to [`w5500-hl`].
//! * `std`: Passthrough to [`w5500-hl`].
//! * `defmt`: Enable logging with `defmt`. Also a passthrough to [`w5500-hl`].
//! * `log`: Enable logging with `log`.
//! * `p256-cm4`: Use [`p256-cm4`], a P256 implementation optimized for the
//!   Cortex-M4 CPU.
//!
//! [`w5500-hl`]: https://github.com/newAM/w5500-hl-rs
//! [`p256-cm4`]: https://crates.io/crates/p256-cm4
//! [Wiznet W5500]: https://www.wiznet.io/product-item/w5500/
#![cfg_attr(docsrs, feature(doc_cfg), feature(doc_auto_cfg))]
#![cfg_attr(all(not(feature = "std"), not(test)), no_std)]
#![deny(unsafe_code)]
#![warn(missing_docs)]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

mod alert;
mod cipher_suites;
mod crypto;
mod extension;
mod handshake;
mod io;
mod key_schedule;
mod record;

use crate::crypto::p256::PublicKey;
pub use alert::{Alert, AlertDescription, AlertLevel};
use core::{cmp::min, convert::Infallible};
use extension::ExtensionType;
use handshake::{
    client_hello::{self, NamedGroup},
    HandshakeType,
};
use hl::{
    io::{Read, Seek, Write},
    ll::{BufferSize, Registers, Sn, SocketInterrupt, SocketInterruptMask},
    net::SocketAddrV4,
    Common, Error as HlError, Hostname, Tcp, TcpReader, TcpWriter,
};
use io::Buffer;
pub use io::{TlsReader, TlsWriter};
use key_schedule::KeySchedule;
pub use rand_core;
use rand_core::{CryptoRng, RngCore};
use record::{ContentType, RecordHeader};
use sha2::{
    digest::{generic_array::GenericArray, typenum::U32},
    Sha256,
};
pub use w5500_hl as hl;
pub use w5500_hl::ll;

const GCM_TAG_LEN: usize = 16;

#[repr(u16)]
enum TlsVersion {
    V1_2 = 0x0303,
    V1_3 = 0x0304,
}

impl From<TlsVersion> for u16 {
    #[inline]
    fn from(tls_version: TlsVersion) -> Self {
        tls_version as u16
    }
}

impl TlsVersion {
    pub const fn msb(self) -> u8 {
        ((self as u16) >> 8) as u8
    }

    pub const fn lsb(self) -> u8 {
        self as u8
    }
}

/// TLS errors.
///
/// When an error occurs the connection is either reset or disconnecting.
///
/// After the connection has disconnected the next call to [`Client::process`]
/// will create a new connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// Alert sent from the server.
    Server(Alert),
    /// Alert sent by the client.
    Client(Alert),
    /// Unexpected TCP disconnection.
    UnexpectedDisconnect,
    /// TCP connection timeout.
    TcpTimeout,
    /// A timeout occurred while waiting for the client to transition from this
    /// state.
    StateTimeout(State),
    /// Tried to write with [`Client::writer`] or [`Client::write_all`] before
    /// the handshake has completed.
    NotConnected,
}

/// Duration in seconds to wait for the TLS server to send a response.
const TIMEOUT_SECS: u32 = 10;

/// Internal TLS client states.
// https://datatracker.ietf.org/doc/html/rfc8446#appendix-A.1
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum State {
    /// Reset and idle.
    Reset,
    /// TCP handshake started, waiting for the CON int.
    WaitConInt,
    /// Sent ClientHello, waiting for ServerHello.
    WaitServerHello,
    /// Received ServerHello, waiting for EncryptedExtensions.
    WaitEncryptedExtensions,
    /// Received EncryptedExtensions, waiting for ServerFinished.
    WaitFinished,
    /// Client will send ClientFinished on the next call to [`Client::process`].
    SendFinished,
    /// Sent ClientFinished, TLS handshake has completed.
    Connected,
    /// The client sent an alert, waiting for the SENDOK interrupt before
    /// starting a TCP disconnection.
    WaitAlertSendOk,
    /// Client will start a TCP disconnection on the next call to
    /// [`Client::process`].
    SendDiscon,
    /// Client started a TCP disconnection, waiting for the DISCON interrupt.
    WaitDiscon,
}

/// TLS events.
///
/// These are events that need to be handled externally by your firmware,
/// such as new application data.
///
/// This is returned by [`Client::process`].
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Event {
    /// A hint to call [`Client::process`] after this many seconds have elapsed.
    ///
    /// This is just a hint and does not have to be used.
    ///
    /// The inner value may increase or decreases with successive calls to
    /// [`Client::process`].
    ///
    /// This is used for state timeout tracking.
    CallAfter(u32),
    /// New application data was received.
    ///
    /// Calling [`Client::reader`] will return a [`TlsReader`] to read the
    /// data.
    ApplicationData,
    /// The handshake finished, and you can read and write application data.
    HandshakeFinished,
    /// Expected disconnection.
    Disconnect,
    /// No event occurred, the client ready and idle.
    None,
}

/// TLS Client.
///
/// # RX Buffer
///
/// The generic `N` is the size of the RX buffer, this must be set to a valid
/// socket [`BufferSize`].
///
/// This buffer must be large enough to contain the largest handshake fragment.
/// The socket RX buffer size will be set to match N.
/// When using pre-shared keys the default value of `N=2048` is typically
/// sufficient.
///
/// This buffer is necessary because handshakes may be fragmented across
/// multiple records, and due to the gaps left by the headers and footers is is
/// not feasible to reassemble fragments within the socket buffers.
pub struct Client<'hn, 'psk, 'b, const N: usize> {
    sn: Sn,
    src_port: u16,
    hostname: Hostname<'hn>,
    dst: SocketAddrV4,
    state: State,

    /// Timeout for TLS server responses
    timeout: Option<u32>,
    key_schedule: KeySchedule,

    identity: &'psk [u8],
    psk: &'psk [u8],

    // RX buffer
    rx: Buffer<'b, N>,
}

const fn size_to_buffersize(size: usize) -> BufferSize {
    match size {
        1024 => BufferSize::KB1,
        2048 => BufferSize::KB2,
        4096 => BufferSize::KB4,
        8192 => BufferSize::KB8,
        16384 => BufferSize::KB16,
        _ => ::core::panic!("valid buffer sizes are 1024, 2048, 4096, 8192, or 16384"),
    }
}

impl<'hn, 'psk, 'b, const N: usize> Client<'hn, 'psk, 'b, N> {
    const RX_BUFFER_SIZE: BufferSize = size_to_buffersize(N);

    // maximum plaintext size
    // https://www.rfc-editor.org/rfc/rfc8449
    // minus 1 because the local memory circular buffer implementation
    // does not use full/empty flags
    const RECORD_SIZE_LIMIT: u16 =
        (N as u16) - (GCM_TAG_LEN as u16) - (RecordHeader::LEN as u16) - 1;

    /// Create a new TLS client.
    ///
    /// You must resolve the hostname to an [`Ipv4Addr`] externally.
    ///
    /// # Arguments
    ///
    /// * `sn` Socket number for the TLS client.
    /// * `src_port` Source port, use any unused port.
    /// * `hostname` Server hostname.
    /// * `dst` Server address.
    /// * `identity` PSK identity
    /// * `psk` pre-shared key
    /// * `rx` RX buffer, this must be 1024, 2048, 4096, 8192, or 16384 bytes
    ///   in length
    ///
    /// # Example
    ///
    /// ```
    /// # const MY_KEY: [u8; 1] = [0];
    /// use w5500_tls::{
    ///     Client,
    ///     {
    ///         hl::Hostname,
    ///         ll::{
    ///             net::{Ipv4Addr, SocketAddrV4},
    ///             Sn,
    ///         },
    ///     },
    /// };
    ///
    /// static mut RX: [u8; 2048] = [0; 2048];
    ///
    /// const DST: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(192, 168, 0, 4), 8883);
    /// const HOSTNAME: Hostname = Hostname::new_unwrapped("server.local");
    /// const SRC_PORT: u16 = 1234;
    /// const TLS_SN: Sn = Sn::Sn4;
    ///
    /// let tls_client: Client<2048> = Client::new(
    ///     TLS_SN,
    ///     SRC_PORT,
    ///     HOSTNAME,
    ///     DST,
    ///     b"mykeyidentity",
    ///     &MY_KEY,
    ///     unsafe { &mut RX },
    /// );
    /// ```
    ///
    /// [`Ipv4Addr`]: w5500_hl::ll::net::Ipv4Addr
    pub fn new(
        sn: Sn,
        src_port: u16,
        hostname: Hostname<'hn>,
        dst: SocketAddrV4,
        identity: &'psk [u8],
        psk: &'psk [u8],
        rx: &'b mut [u8; N],
    ) -> Self {
        Self {
            sn,
            src_port,
            hostname,
            dst,
            state: State::Reset,
            timeout: None,
            key_schedule: KeySchedule::default(),
            identity,
            psk,
            rx: Buffer::from(rx),
        }
    }

    fn timeout_elapsed_secs(&self, monotonic_secs: u32) -> Option<u32> {
        self.timeout.map(|to| monotonic_secs - to)
    }

    fn set_state_with_timeout(&mut self, state: State, monotonic_secs: u32) -> u32 {
        debug!(
            "{:?} -> {:?} with timeout {}",
            self.state, state, monotonic_secs
        );
        self.state = state;
        self.timeout = Some(monotonic_secs);
        TIMEOUT_SECS
    }

    fn set_state(&mut self, state: State) {
        debug!("{:?} -> {:?} without timeout", self.state, state);
        self.state = state;
        self.timeout = None;
    }

    fn set_state_send_discon(&mut self, monotonic_secs: u32) -> u32 {
        self.key_schedule.reset();
        self.set_state_with_timeout(State::SendDiscon, monotonic_secs)
    }

    fn reset(&mut self) {
        self.key_schedule.reset();
        self.set_state(State::Reset);
    }

    /// Process the MQTT client.
    ///
    /// This should be called repeatedly until it returns:
    ///
    /// * `Err(_)` What to do upon errors is up to you.
    /// * `Ok(Event::CallAfter(seconds))` Call this method again after the number
    ///   of seconds indicated.
    /// * `Ok(Event::None)` The client is idle; you can call [`writer`](Self::writer).
    ///
    /// This should also be called when there is a pending socket interrupt.
    ///
    /// # Arguments
    ///
    /// * `w5500` W5500 device implementing the [`Registers`] trait.
    /// * `rng` secure random number generator.
    ///   This is assumed to be infallible.
    ///   If you have a fallible secure hardware RNG you can use that to seed
    ///   an infallible software RNG.
    /// * `monotonic_secs` Monotonically increasing (never decreasing) seconds
    ///   since an epoch (typically system boot).
    pub fn process<'w, W5500: Registers, R: RngCore + CryptoRng>(
        &mut self,
        w5500: &'w mut W5500,
        rng: &mut R,
        monotonic_secs: u32,
    ) -> Result<Event, Error> {
        let sn_ir: SocketInterrupt = w5500.sn_ir(self.sn).unwrap_or_default();

        if sn_ir.any_raised() {
            if w5500.set_sn_ir(self.sn, sn_ir).is_err() {
                return Err(self.send_fatal_alert(
                    w5500,
                    AlertDescription::InternalError,
                    monotonic_secs,
                ));
            }

            if sn_ir.con_raised() {
                info!("CONN interrupt");
                if let Err(e) = self.send_client_hello(w5500, rng, monotonic_secs) {
                    return Err(self.send_fatal_alert(w5500, e, monotonic_secs));
                }
            }
            if sn_ir.discon_raised() {
                info!("DISCON interrupt");
                // TODO: try to get discon reason from server
                if self.state != State::WaitDiscon {
                    warn!("Unexpected TCP disconnect");
                    self.reset();
                    return Ok(Event::Disconnect);
                } else {
                    return Err(Error::UnexpectedDisconnect);
                }
            }
            if sn_ir.recv_raised() {
                info!("RECV interrupt");
            }
            if sn_ir.timeout_raised() {
                info!("TIMEOUT interrupt");
                self.reset();
                return Err(Error::TcpTimeout);
            }
            if sn_ir.sendok_raised() {
                info!("SENDOK interrupt");
                if self.state == State::WaitAlertSendOk {
                    return Ok(Event::CallAfter(self.set_state_send_discon(monotonic_secs)));
                }
            }
        }

        match self.state {
            State::Reset => {
                match self.tcp_connect(w5500, monotonic_secs) {
                    Ok(after) => return Ok(Event::CallAfter(after)),
                    Err(e) => return Err(self.send_fatal_alert(w5500, e, monotonic_secs)),
                };
            }
            State::SendDiscon => {
                if w5500.tcp_disconnect(self.sn).is_err() {
                    return Err(self.send_fatal_alert(
                        w5500,
                        AlertDescription::InternalError,
                        monotonic_secs,
                    ));
                }
                let after: u32 = self.set_state_with_timeout(State::WaitDiscon, monotonic_secs);
                return Ok(Event::CallAfter(after));
            }
            _ => (),
        }

        // all incoming data must be ignored after sending an alert
        if !matches!(self.state, State::WaitAlertSendOk | State::WaitDiscon) {
            let sn_rx_rsr: u16 = match w5500.sn_rx_rsr(self.sn) {
                Ok(sn_rx_rsr) => sn_rx_rsr,
                Err(_) => {
                    return Err(self.send_fatal_alert(
                        w5500,
                        AlertDescription::InternalError,
                        monotonic_secs,
                    ))
                }
            };
            if sn_rx_rsr >= RecordHeader::LEN as u16 {
                if let Some(event) = self.recv(w5500, monotonic_secs)? {
                    return Ok(event);
                }
            }

            if matches!(self.state, State::SendFinished) {
                if let Err(e) = self.send_client_finished(w5500) {
                    return Err(self.send_fatal_alert(w5500, e, monotonic_secs));
                }
                return Ok(Event::HandshakeFinished);
            }
        }

        if let Some(elapsed_secs) = self.timeout_elapsed_secs(monotonic_secs) {
            if elapsed_secs > TIMEOUT_SECS {
                info!(
                    "timeout waiting for state to transition from {:?}",
                    self.state
                );
                let ret = Err(Error::StateTimeout(self.state));
                if matches!(self.state, State::WaitDiscon) {
                    self.reset()
                } else {
                    self.set_state(State::SendDiscon);
                }
                ret
            } else {
                let call_after: u32 = TIMEOUT_SECS.saturating_sub(elapsed_secs);
                Ok(Event::CallAfter(call_after))
            }
        } else {
            Ok(Event::None)
        }
    }

    fn tcp_connect<W5500: Registers>(
        &mut self,
        w5500: &mut W5500,
        monotonic_secs: u32,
    ) -> Result<u32, AlertDescription> {
        debug!("connecting to {}", self.dst);
        w5500
            .close(self.sn)
            .map_err(|_| AlertDescription::InternalError)?;
        w5500
            .set_sn_rxbuf_size(self.sn, Self::RX_BUFFER_SIZE)
            .map_err(|_| AlertDescription::InternalError)?;
        let simr: u8 = w5500.simr().map_err(|_| AlertDescription::InternalError)?;
        w5500
            .set_simr(self.sn.bitmask() | simr)
            .map_err(|_| AlertDescription::InternalError)?;
        w5500
            .set_sn_imr(self.sn, SocketInterruptMask::DEFAULT)
            .map_err(|_| AlertDescription::InternalError)?;
        w5500
            .tcp_connect(self.sn, self.src_port, &self.dst)
            .map_err(|_| AlertDescription::InternalError)?;
        Ok(self.set_state_with_timeout(State::WaitConInt, monotonic_secs))
    }

    /// ```text
    /// struct {
    ///     ProtocolVersion legacy_version = 0x0303;    /* TLS v1.2 */
    ///     Random random;
    ///     opaque legacy_session_id<0..32>;
    ///     CipherSuite cipher_suites<2..2^16-2>;
    ///     opaque legacy_compression_methods<1..2^8-1>;
    ///     Extension extensions<8..2^16-1>;
    /// } ClientHello;
    /// ```
    fn send_client_hello<W5500: Registers, R: RngCore + CryptoRng>(
        &mut self,
        w5500: &mut W5500,
        rng: &mut R,
        monotonic_secs: u32,
    ) -> Result<(), AlertDescription> {
        self.rx.reset();

        let mut random: [u8; 32] = [0; 32];
        rng.fill_bytes(&mut random);

        #[cfg(feature = "std")]
        self.key_schedule.client_random.replace(random);

        let client_public_key = self.key_schedule.new_client_secret(rng);

        // using fragment buffer for TX since it is unused at this point
        let len: usize = client_hello::ser(
            self.rx.as_mut_buf(),
            &random,
            &self.hostname,
            &client_public_key,
            &mut self.key_schedule,
            self.psk,
            self.identity,
            Self::RECORD_SIZE_LIMIT,
        );
        let buf: &[u8] = &self.rx.as_buf()[..len];

        let mut writer: TcpWriter<W5500> = w5500
            .tcp_writer(self.sn)
            .map_err(|_| AlertDescription::InternalError)?;
        writer
            .write_all(buf)
            .map_err(|_| AlertDescription::InternalError)?;
        writer.send().map_err(|_| AlertDescription::InternalError)?;

        self.key_schedule.increment_write_record_sequence_number();
        self.set_state_with_timeout(State::WaitServerHello, monotonic_secs);
        self.key_schedule.initialize_early_secret();

        Ok(())
    }

    /// Send an alert to the server.
    ///
    /// # References
    ///
    /// * [RFC 8446 Appendix B.2](https://datatracker.ietf.org/doc/html/rfc8446#appendix-B.2)
    ///
    /// ```text
    /// struct {
    ///     AlertLevel level;
    ///     AlertDescription description;
    /// } Alert;
    /// ```
    fn send_alert<W5500: Registers>(
        &mut self,
        w5500: &mut W5500,
        level: AlertLevel,
        description: AlertDescription,
        monotonic_secs: u32,
    ) {
        debug!("send_alert {:?} {:?}", level, description);

        let mut try_send_alert = || -> Result<(), AlertDescription> {
            if self.key_schedule.server_traffic_secret_exists() {
                self.send_encrypted_record(
                    w5500,
                    ContentType::Alert,
                    &[level.into(), description.into()],
                )
                .map_err(AlertDescription::map_w5500)?;
            } else {
                #[rustfmt::skip]
                let buf: [u8; 7] = [
                    ContentType::Alert.into(),
                    TlsVersion::V1_2.msb(),
                    TlsVersion::V1_2.lsb(),
                    0, 2, // length
                    level.into(),
                    description.into(),
                ];
                let mut writer: TcpWriter<W5500> = w5500
                    .tcp_writer(self.sn)
                    .map_err(|_| AlertDescription::InternalError)?;
                writer
                    .write_all(&buf)
                    .map_err(AlertDescription::map_w5500)?;
                writer.send().map_err(|_| AlertDescription::InternalError)?;
            }
            Ok(())
        };

        let result: Result<(), AlertDescription> = try_send_alert();

        self.key_schedule.reset();

        if let Err(e1) = result {
            error!("error while sending alert: {:?}", e1);
            self.set_state_send_discon(monotonic_secs);
        } else {
            self.key_schedule.reset();
            self.set_state_with_timeout(State::WaitAlertSendOk, monotonic_secs);
        }
    }

    fn send_fatal_alert<W5500: Registers>(
        &mut self,
        w5500: &mut W5500,
        description: AlertDescription,
        monotonic_secs: u32,
    ) -> Error {
        self.send_alert(w5500, AlertLevel::Fatal, description, monotonic_secs);
        Error::Client(Alert::new_fatal(description))
    }

    fn recv_change_cipher_spec(&mut self, header: &RecordHeader) -> Result<(), AlertDescription> {
        if header.length() != 1 {
            error!(
                "expected length 1 for ChangeCipherSpec got {}",
                header.length()
            );
            Err(AlertDescription::DecodeError)
        } else {
            let value: u8 = self.rx.pop_tail().ok_or(AlertDescription::DecodeError)?;

            // https://datatracker.ietf.org/doc/html/rfc8446#section-5
            // An implementation may receive an unencrypted record of type
            // change_cipher_spec consisting of the single byte value 0x01 at any
            // time after the first ClientHello message has been sent or received
            // and before the peer's Finished message has been received and MUST
            // simply drop it without further processing.
            //
            // An implementation which receives any other change_cipher_spec value or
            // which receives a protected change_cipher_spec record MUST abort the
            // handshake with an "unexpected_message" alert.
            const REQUIRED_VALUE: u8 = 0x01;
            if value != REQUIRED_VALUE {
                error!(
                    "change_cipher_spec value {:#02X} does not match expected value {:#02X}",
                    value, REQUIRED_VALUE
                );
                Err(AlertDescription::UnexpectedMessage)
            } else {
                Ok(())
            }
        }
    }

    fn recv_header<W5500: Registers>(
        &self,
        w5500: &mut W5500,
    ) -> Result<Option<RecordHeader>, AlertDescription> {
        let mut header_buf: [u8; 5] = [0; 5];

        let mut reader: TcpReader<W5500> = w5500
            .tcp_reader(self.sn)
            .map_err(AlertDescription::map_w5500)?;
        reader
            .read_exact(&mut header_buf)
            .map_err(AlertDescription::map_w5500)?;

        let header: RecordHeader = RecordHeader::deser(header_buf)?;
        debug!("RecordHeader.length={}", header.length());

        // The length MUST NOT exceed 2^14 bytes.
        // An endpoint that receives a record that exceeds this length MUST
        // terminate the connection with a "record_overflow" alert.
        //
        // We use the record size limit extension, so we can limit this to
        // our RX buffer size
        if header.length() > Self::RECORD_SIZE_LIMIT {
            Err(AlertDescription::RecordOverflow)
        } else if header.length().saturating_add(RecordHeader::LEN as u16) > reader.stream_len() {
            Ok(None)
        } else {
            reader.done().map_err(|_| AlertDescription::InternalError)?;
            Ok(Some(header))
        }
    }

    fn recv_unencrypted_body<W5500: Registers>(
        &mut self,
        w5500: &mut W5500,
        header: &RecordHeader,
    ) -> Result<(), AlertDescription> {
        let mut reader: TcpReader<W5500> = w5500
            .tcp_reader(self.sn)
            .map_err(AlertDescription::map_w5500)?;
        let mut remain: usize = header.length().into();
        let mut buf: [u8; 64] = [0; 64];
        loop {
            let read_len: usize = min(remain, buf.len());
            if read_len == 0 {
                break;
            }
            reader
                .read_exact(&mut buf[..read_len])
                .map_err(AlertDescription::map_w5500)?;
            self.rx.extend_from_slice(&buf[..read_len])?;
            remain -= read_len;
        }

        reader.done().map_err(|_| AlertDescription::InternalError)?;
        Ok(())
    }

    fn recv<W5500: Registers>(
        &mut self,
        w5500: &mut W5500,
        monotonic_secs: u32,
    ) -> Result<Option<Event>, Error> {
        let header: RecordHeader = match self.recv_header(w5500) {
            Ok(Some(header)) => header,
            Ok(None) => return Ok(None),
            Err(e) => return Err(self.send_fatal_alert(w5500, e, monotonic_secs)),
        };

        let rx_buffer_contains_handshake_fragment: bool = self.rx.contains_handshake_fragment();

        let actual_content_type: ContentType =
            if matches!(header.content_type(), ContentType::ApplicationData) {
                debug!("decrypting record");

                let (key, nonce): ([u8; 16], [u8; 12]) =
                    match self.key_schedule.server_key_and_nonce() {
                        Some(x) => x,
                        None => {
                            error!("received ApplicationData before establishing keys");
                            return Err(self.send_fatal_alert(
                                w5500,
                                AlertDescription::UnexpectedMessage,
                                monotonic_secs,
                            ));
                        }
                    };

                match crypto::decrypt_record_inplace(
                    w5500,
                    self.sn,
                    &key,
                    &nonce,
                    &header,
                    &mut self.rx,
                ) {
                    Ok(Ok(content_type)) => content_type,
                    Ok(Err(x)) => {
                        error!("ContentType {:02X}", x);
                        return Err(self.send_fatal_alert(
                            w5500,
                            AlertDescription::DecodeError,
                            monotonic_secs,
                        ));
                    }
                    Err(e) => return Err(self.send_fatal_alert(w5500, e, monotonic_secs)),
                }
            } else {
                if let Err(e) = self.recv_unencrypted_body(w5500, &header) {
                    return Err(self.send_fatal_alert(w5500, e, monotonic_secs));
                }
                header.content_type()
            };

        debug!("RecordHeader.content_type={:?}", actual_content_type);

        if matches!(actual_content_type, ContentType::ApplicationData) {
            self.rx.increment_application_data_tail(
                header
                    .length()
                    .saturating_sub((GCM_TAG_LEN + 1) as u16)
                    .into(),
            );
        }

        if rx_buffer_contains_handshake_fragment
            && !matches!(actual_content_type, ContentType::Handshake)
        {
            // https://datatracker.ietf.org/doc/html/rfc8446#section-5.1
            error!("Handshake messages MUST NOT be interleaved with other record types");
            return Err(self.send_fatal_alert(
                w5500,
                AlertDescription::UnexpectedMessage,
                monotonic_secs,
            ));
        }

        let ret = match actual_content_type {
            // https://datatracker.ietf.org/doc/html/rfc8446#section-5.1
            // No mention if change_cipher_spec may or may not be fragmented
            // This is such a short ContentType that I will assume that it
            // does not fragment
            ContentType::ChangeCipherSpec => {
                if let Err(e) = self.recv_change_cipher_spec(&header) {
                    Err(self.send_fatal_alert(w5500, e, monotonic_secs))
                } else {
                    Ok(None)
                }
            }
            // "Alert messages MUST NOT be fragmented across records"
            ContentType::Alert => return Err(self.recv_alert(w5500, &header)),
            ContentType::Handshake => {
                if let Err(e) = self.recv_handshake(monotonic_secs) {
                    Err(self.send_fatal_alert(w5500, e, monotonic_secs))
                } else {
                    Ok(None)
                }
            }
            ContentType::ApplicationData => Ok(Some(Event::ApplicationData)),
        };

        if matches!(header.content_type(), ContentType::ApplicationData) {
            self.key_schedule.increment_read_record_sequence_number();
        }

        ret
    }

    fn recv_alert<W5500: Registers>(&mut self, w5500: &mut W5500, header: &RecordHeader) -> Error {
        self.set_state(State::Reset);
        self.key_schedule.reset();

        if header.length() != 2 {
            error!("expected length 2 for Alert got {}", header.length());
            self.rx.reset();
            w5500.tcp_disconnect(self.sn).ok();
            Error::Client(Alert {
                level: AlertLevel::Fatal,
                description: AlertDescription::DecodeError,
            })
        } else {
            let description: AlertDescription = match self.rx.pop_tail() {
                Some(byte) => match AlertDescription::try_from(byte) {
                    Ok(description) => description,
                    Err(e) => {
                        error!("unknown alert description {}", e);
                        return Error::Client(Alert {
                            level: AlertLevel::Fatal,
                            description: AlertDescription::DecodeError,
                        });
                    }
                },
                None => {
                    self.rx.reset();
                    return Error::Client(Alert {
                        level: AlertLevel::Fatal,
                        description: AlertDescription::DecodeError,
                    });
                }
            };

            let level: AlertLevel = match self.rx.pop_tail() {
                Some(byte) => match AlertLevel::try_from(byte) {
                    Ok(level) => level,
                    Err(e) => {
                        error!("illegal alert level {}", e);
                        AlertLevel::Fatal
                    }
                },
                None => {
                    self.rx.reset();
                    return Error::Client(Alert {
                        level: AlertLevel::Fatal,
                        description: AlertDescription::DecodeError,
                    });
                }
            };

            let alert: Alert = Alert { level, description };

            match level {
                AlertLevel::Warning => warn!("{:?}", alert),
                AlertLevel::Fatal => error!("{:?}", alert),
            }

            self.rx.reset();
            Error::Server(alert)
        }
    }

    fn send_client_finished<W5500: Registers>(
        &mut self,
        w5500: &mut W5500,
    ) -> Result<(), AlertDescription> {
        let verify_data: GenericArray<u8, U32> = self.key_schedule.client_finished_verify_data();
        let data: [u8; 36] = handshake::client_finished(&verify_data);

        self.send_encrypted_record(w5500, ContentType::Handshake, &data)
            .map_err(AlertDescription::map_w5500)?;
        self.set_state(State::Connected);

        // master secrets are only ClientHello..server Finished
        // no need to update the key schedule for this.
        self.key_schedule.initialize_master_secret();

        Ok(())
    }

    // helper to send an encrypted record without a round-trip to the socket
    // buffers
    fn send_encrypted_record<W5500: Registers>(
        &mut self,
        w5500: &mut W5500,
        content_type: ContentType,
        data: &[u8],
    ) -> Result<(), HlError<W5500::Error>> {
        const CONTENT_TYPE_LEN: usize = 1;
        let data_len: u16 = unwrap!((data.len() + GCM_TAG_LEN + CONTENT_TYPE_LEN).try_into());

        let header: [u8; 5] = [
            ContentType::ApplicationData.into(),
            TlsVersion::V1_2.msb(),
            TlsVersion::V1_2.lsb(),
            (data_len >> 8) as u8,
            data_len as u8,
        ];

        let mut writer: TcpWriter<W5500> = w5500.tcp_writer(self.sn)?;

        // write the record header
        writer.write_all(&header)?;

        let (key, nonce): ([u8; 16], [u8; 12]) = self.key_schedule.client_key_and_nonce().unwrap();
        let mut cipher = crate::crypto::Aes128Gcm::new(&key, &nonce, &header);

        // write the record data in 128-bit chunks
        let mut chunks = data.chunks_exact(16);
        for chunk in &mut chunks {
            let mut mut_chunck: [u8; 16] = chunk.try_into().unwrap();
            cipher.encrypt_block_inplace(&mut mut_chunck);
            writer.write_all(&mut_chunck)?;
        }

        // write the remaining data
        let rem = chunks.remainder();
        let mut padded_block: [u8; 16] = [0; 16];
        padded_block[..rem.len()].copy_from_slice(rem);
        // append the content type
        padded_block[rem.len()] = content_type as u8;
        let remainder_len: usize = rem.len() + CONTENT_TYPE_LEN;
        cipher.encrypt_remainder_inplace(&mut padded_block, remainder_len);
        writer.write_all(&padded_block[..remainder_len])?;

        // write the AES-GCM authentication tag
        let tag: [u8; GCM_TAG_LEN] = cipher.finish();
        writer.write_all(&tag)?;
        writer.send()?;

        Ok(())
    }

    fn recv_handshake(&mut self, monotonic_secs: u32) -> Result<(), AlertDescription> {
        loop {
            let mut hash: Sha256 = self.key_schedule.transcript_hash();
            let (header, mut reader) = match self.rx.pop_handshake_record(&mut hash)? {
                // fragment is not long enough to contain handshake type + length
                None => return Ok(()),
                Some(s) => s,
            };

            match header.msg_type() {
                Ok(HandshakeType::ClientHello) => {
                    error!("unexpected ClientHello");
                    return Err(AlertDescription::UnexpectedMessage);
                }
                Ok(HandshakeType::ServerHello) => {
                    if self.state != State::WaitServerHello {
                        error!("unexpected ServerHello in state {:?}", self.state);
                        return Err(AlertDescription::UnexpectedMessage);
                    } else {
                        let public_key: PublicKey = handshake::recv_server_hello(&mut reader)?;

                        self.key_schedule.set_server_public_key(public_key);
                        self.key_schedule.set_transcript_hash(hash.clone());
                        self.key_schedule.initialize_handshake_secret();
                        self.set_state_with_timeout(State::WaitEncryptedExtensions, monotonic_secs);
                    }
                }
                Ok(HandshakeType::NewSessionTicket) => {
                    if self.state != State::Connected {
                        error!("unexpected NewSessionTicket in state {:?}", self.state);
                        return Err(AlertDescription::UnexpectedMessage);
                    } else {
                        // https://datatracker.ietf.org/doc/html/rfc8446#section-4.6.1
                        // At any time after the server has received the client Finished
                        // message, it MAY send a NewSessionTicket message.
                        // The client MAY use this PSK for future handshakes by including the
                        // ticket value in the "pre_shared_key" extension in its ClientHello
                        info!("NewSessionTicket is unused");
                    }
                }
                Ok(HandshakeType::EndOfEarlyData) => {
                    // should never occur unless we support PSK
                    // https://datatracker.ietf.org/doc/html/rfc8446#section-4.2.10
                    error!("PSK is not supported");
                    return Err(AlertDescription::UnexpectedMessage);
                }
                Ok(HandshakeType::EncryptedExtensions) => {
                    if self.state != State::WaitEncryptedExtensions {
                        error!("unexpected Certificate in state {:?}", self.state);
                        return Err(AlertDescription::UnexpectedMessage);
                    }

                    handshake::recv_encrypted_extensions(&mut reader)?;
                    self.set_state_with_timeout(State::WaitFinished, monotonic_secs);
                }
                Ok(
                    hs_type @ (HandshakeType::Certificate
                    | HandshakeType::CertificateRequest
                    | HandshakeType::CertificateVerify),
                ) => {
                    error!(
                        "unexpected extension {:?} certificate authentication not supported",
                        hs_type
                    );
                    return Err(AlertDescription::UnexpectedMessage);
                }
                Ok(HandshakeType::Finished) => {
                    if self.state != State::WaitFinished {
                        error!("unexpected Finished in state {:?}", self.state);
                        return Err(AlertDescription::UnexpectedMessage);
                    }

                    const VERIFY_DATA_LEN: usize = 32;
                    if header.length() != VERIFY_DATA_LEN as u32 {
                        error!(
                            "expected verify_data length {} got {}",
                            VERIFY_DATA_LEN,
                            header.length()
                        );
                        return Err(AlertDescription::UnexpectedMessage);
                    }

                    let verify_data: [u8; 32] = reader.next_n()?;
                    self.key_schedule.verify_server_finished(&verify_data)?;
                    self.set_state_with_timeout(State::SendFinished, monotonic_secs);
                }

                Ok(HandshakeType::KeyUpdate) => {
                    if self.state != State::Connected {
                        // https://datatracker.ietf.org/doc/html/rfc8446#section-4.6.3
                        // Implementations that receive a KeyUpdate message prior to
                        // receiving a Finished message MUST terminate the connection
                        // with an "unexpected_message"
                        error!("unexpected KeyUpdate in state {:?}", self.state);
                        return Err(AlertDescription::UnexpectedMessage);
                    }

                    const EXPECTED_LEN: u32 = 1;
                    if header.length() != EXPECTED_LEN {
                        error!(
                            "expected KeyUpdate length {} got {}",
                            EXPECTED_LEN,
                            header.length()
                        );
                        return Err(AlertDescription::UnexpectedMessage);
                    }

                    match handshake::KeyUpdateRequest::try_from(reader.next_u8()?) {
                        Ok(handshake::KeyUpdateRequest::UpdateNotRequested) => {
                            // should never occur because we never request a key update
                            warn!("KeyUpdate without update_requested");
                        }
                        Ok(handshake::KeyUpdateRequest::UpdateRequested) => {
                            warn!("TODO update_traffic_secret is untested");
                            self.key_schedule.update_traffic_secret();
                        }
                        Err(x) => {
                            error!("illegal KeyUpdateRequest value: 0x{:02X}", x);
                            return Err(AlertDescription::IllegalParameter);
                        }
                    }
                }
                Err(x) => {
                    warn!("invalid msg_type {:?}", x);
                    return Err(AlertDescription::UnexpectedMessage);
                }
            }

            self.key_schedule.set_transcript_hash(hash);
        }
    }

    /// Returns `true` if the TLS handshake has completed and the client is
    /// connected.
    ///
    /// # Example
    ///
    /// ```
    /// # const MY_KEY: [u8; 1] = [0];
    /// use w5500_tls::{
    ///     Client,
    ///     {
    ///         hl::Hostname,
    ///         ll::{
    ///             net::{Ipv4Addr, SocketAddrV4},
    ///             Sn,
    ///         },
    ///     },
    /// };
    ///
    /// static mut RX: [u8; 2048] = [0; 2048];
    ///
    /// const DST: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(192, 168, 0, 4), 8883);
    /// const HOSTNAME: Hostname = Hostname::new_unwrapped("server.local");
    /// const SRC_PORT: u16 = 1234;
    /// const TLS_SN: Sn = Sn::Sn4;
    ///
    /// let tls_client: Client<2048> = Client::new(
    ///     TLS_SN,
    ///     SRC_PORT,
    ///     HOSTNAME,
    ///     DST,
    ///     b"mykeyidentity",
    ///     &MY_KEY,
    ///     unsafe { &mut RX },
    /// );
    //
    // assert_eq!(tls_client.connected(), false);
    // ```
    pub fn connected(&self) -> bool {
        self.state == State::Connected
    }

    /// Create a TLS writer.
    ///
    /// This returns a [`TlsWriter`] structure, which contains functions to
    /// stream data to the W5500 socket buffers incrementally.
    ///
    /// This is similar to [`TcpWriter`], except it will encrypt the data before
    /// sending.
    ///
    /// This is slower than [`write_all`](Self::write_all), it will
    /// write all your data, read it back, encrypt it, then write it back
    /// before sending.  This is useful for low-memory applications.
    ///
    /// # Errors
    ///
    /// This method can only return:
    ///
    /// * [`Error::Client`] with [`AlertDescription::InternalError`]
    /// * [`Error::NotConnected`]
    ///
    /// # Example
    ///
    /// See [`TlsWriter`].
    pub fn writer<'w, 'ks, W5500: Registers>(
        &'ks mut self,
        w5500: &'w mut W5500,
    ) -> Result<TlsWriter<'w, 'ks, W5500>, Error>
    where
        Self: Sized,
    {
        const TRAILING_CONTENT_TYPE_LEN: u16 = 1;
        const RECORD_HEADER_LEN: u16 = RecordHeader::LEN as u16;
        const TLS_OVERHEAD: u16 =
            RECORD_HEADER_LEN + (GCM_TAG_LEN as u16) + TRAILING_CONTENT_TYPE_LEN;

        if !self.connected() {
            return Err(Error::NotConnected);
        }

        // if there is not enough space for the TLS overhead return an error
        let sn_tx_fsr: u16 = w5500
            .sn_tx_fsr(self.sn)
            .map_err(|_| Error::Client(Alert::new_warning(AlertDescription::InternalError)))?
            .checked_sub(TLS_OVERHEAD)
            .ok_or_else(|| Error::Client(Alert::new_warning(AlertDescription::InternalError)))?;

        // advance write pointer by 5 to leave room for the record header
        let sn_tx_wr: u16 = w5500
            .sn_tx_wr(self.sn)
            .map_err(|_| Error::Client(Alert::new_warning(AlertDescription::InternalError)))?
            .wrapping_add(RECORD_HEADER_LEN);

        Ok(TlsWriter {
            w5500,
            key_schedule: &mut self.key_schedule,
            sn: self.sn,
            head_ptr: sn_tx_wr,
            tail_ptr: sn_tx_wr.wrapping_add(sn_tx_fsr),
            ptr: sn_tx_wr,
        })
    }

    /// Send data to the remote host.
    ///
    /// This is more efficient than [`writer`](Self::writer) because the data
    /// size is known up-front and a round-trip to the socket buffers to
    /// encrypt the record can be avoided.
    ///
    /// This should only be used when the handshake has completed, otherwise
    /// the server will send an `unexpected_message` alert.
    ///
    /// # Errors
    ///
    /// This method can only return:
    ///
    /// * [`Error::Client`] with [`AlertDescription::InternalError`]
    /// * [`Error::NotConnected`]
    pub fn write_all<W5500: Registers>(
        &mut self,
        w5500: &mut W5500,
        data: &[u8],
    ) -> Result<(), Error> {
        if !self.connected() {
            Err(Error::NotConnected)
        } else {
            self.send_encrypted_record(w5500, ContentType::ApplicationData, data)
                .map_err(|_| Error::Client(Alert::new_warning(AlertDescription::InternalError)))
        }
    }

    /// Create a TLS reader.
    ///
    /// # Errors
    ///
    /// This method can only return:
    ///
    /// * [`HlError::Other`]
    /// * [`HlError::WouldBlock`]
    ///
    /// # Example
    ///
    /// See [`TlsReader`].
    pub fn reader<'ptr>(&'ptr mut self) -> Result<TlsReader<'b, 'ptr>, HlError<Infallible>> {
        self.rx.app_data_reader()
    }
}
