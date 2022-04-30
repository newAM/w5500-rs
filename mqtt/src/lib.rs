//! MQTT v5 client for the [Wiznet W5500] SPI internet offload chip.
//!
//! # Limitations
//!
//! This is very basic at the moment, and will be expanded in the future.
//!
//! * Does not support TLS.
//! * Does not support password protected MQTT servers.
//! * Only supports QoS 0: At most once delivery.
//!
//! # Example
//!
//! ```no_run
//! # fn monotonic_secs() -> u32 { 0 }
//! # let mut w5500 = w5500_regsim::W5500::default();
//! use w5500_mqtt::{
//!     ll::{
//!         net::{Ipv4Addr, SocketAddrV4},
//!         Sn,
//!     },
//!     Client, ClientId, Event, DST_PORT, SRC_PORT,
//! };
//!
//! let mut client: Client = Client::new(
//!     Sn::Sn2,
//!     SRC_PORT,
//!     SocketAddrV4::new(Ipv4Addr::new(192, 168, 5, 6), DST_PORT),
//! );
//!
//! // wait for a connection or die trying
//! while client.process(&mut w5500, monotonic_secs())? != Event::None {}
//!
//! // publish to "duck" with a payload "quack"
//! client.publish(&mut w5500, "duck", b"quack")?;
//!
//! // subscribe to "cow"
//! client.subscribe(&mut w5500, "cow")?;
//! # Ok::<(), w5500_mqtt::Error<std::io::ErrorKind>>(())
//! ```
//!
//! # Relevant Specifications
//!
//! * [MQTT Version 5.0](https://docs.oasis-open.org/mqtt/mqtt/v5.0/mqtt-v5.0.html)
//!
//! # Feature Flags
//!
//! All features are disabled by default.
//!
//! * `embedded-hal`: Passthrough to [w5500-hl].
//! * `std`: Passthrough to [w5500-hl].
//! * `defmt`: Enable logging with `defmt`. Also a passthrough to [w5500-hl].
//! * `log`: Enable logging with `log`.
//! * `w5500-tls`: Enable MQTT over TLS with pre-shared keys.
//!
//! [w5500-hl]: https://crates.io/crates/w5500-hl
//! [Wiznet W5500]: https://www.wiznet.io/product-item/w5500/
#![cfg_attr(docsrs, feature(doc_cfg), feature(doc_auto_cfg))]
#![cfg_attr(all(not(feature = "std"), not(test)), no_std)]
#![deny(unsafe_code)]
#![warn(missing_docs)]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

mod client_id;
mod connect;
mod data;
mod fixed_header;
mod properties;
mod publish;
mod recv;
mod subscribe;

#[cfg(feature = "w5500-tls")]
pub mod tls;

pub use client_id::ClientId;
use connect::send_connect;
pub use connect::ConnectReasonCode;
use hl::{
    io::{Read, Seek, Write},
    ll::{net::SocketAddrV4, Registers, Sn, SocketInterrupt, SocketInterruptMask},
    Error as HlError, Tcp, TcpReader, TcpWriter,
};
use publish::send_publish;
pub use publish::PublishReader;
use recv::recv;
use subscribe::{send_subscribe, send_unsubscribe};
pub use subscribe::{SubAck, SubAckReasonCode, UnSubAck, UnSubAckReasonCode};
pub use w5500_hl as hl;
pub use w5500_hl::ll;

/// Default MQTT destination port.
pub const DST_PORT: u16 = 1883;
/// Default MQTT source port.
pub const SRC_PORT: u16 = 33650;

/// Control packet types.
///
/// [MQTT Control Packet types](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901022)
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[allow(clippy::upper_case_acronyms)]
pub(crate) enum CtrlPkt {
    RESERVED = 0x0,
    CONNECT = 0x1,
    CONNACK = 0x2,
    PUBLISH = 0x3,
    PUBACK = 0x4,
    PUBREC = 0x5,
    PUBREL = 0x6,
    PUBCOMP = 0x7,
    SUBSCRIBE = 0x8,
    SUBACK = 0x9,
    UNSUBSCRIBE = 0xA,
    UNSUBACK = 0xB,
    PINGREQ = 0xC,
    PINGRESP = 0xD,
    DISCONNECT = 0xE,
    AUTH = 0xF,
}

/// Internal MQTT client state.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum State {
    /// W5500 socket is in an unknown state.
    Init,
    /// Socket has been initialized, waiting for an established TCP connection.
    ///
    /// When using TLS this is instead waiting for the TLS handshake to complete.
    WaitConInt,
    /// CONNECT packet has been sent, waiting for a CONNACK.
    WaitConAck,
    /// CONNACK has been received, ready for action.
    Ready,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
struct StateTimeout {
    /// MQTT client state
    state: State,
    /// Timeout for MQTT server responses
    timeout: Option<u32>,
}

impl StateTimeout {
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
}

/// Duration in seconds to wait for the MQTT server to send a response.
const TIMEOUT_SECS: u32 = 10;

fn write_variable_byte_integer<E, Writer: Write<E>>(
    writer: &mut Writer,
    integer: u32,
) -> Result<(), HlError<E>> {
    let (buf, len): ([u8; 4], usize) = crate::data::encode_variable_byte_integer(integer);
    writer.write_all(&buf[..len])
}

/// MQTT events.
///
/// These are events that need to be handled externally by your firmware,
/// such as a published message on a subscribed topic.
///
/// This is returned by [`Client::process`].
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Event<E, Reader: Read<E> + Seek<E>> {
    /// A hint to call [`Client::process`] after this many seconds have elapsed.
    ///
    /// This is just a hint and does not have to be used.
    ///
    /// The inner value may increase or decreases with successive calls to
    /// [`Client::process`].
    ///
    /// This is used for state timeout tracking.
    CallAfter(u32),
    /// A message was published to a subscribed topic.
    ///
    /// The inner value contains a [`PublishReader`] to extract the topic and
    /// payload from the socket buffers.
    Publish(PublishReader<E, Reader>),
    /// Subscribe Acknowledgment.
    SubAck(SubAck),
    /// Unsubscribe Acknowledgment.
    UnSubAck(UnSubAck),
    /// The connection has been accepted by the server.
    ///
    /// This is a good time to subscribe to topics.
    ConnAck,
    /// No event occurred, the client ready and idle.
    None,
}

/// MQTT client errors.
///
/// When an error occurs the client state is reset to [`State::Init`].
/// The next call to [`Client::process`] will create a new connection.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error<E> {
    /// A timeout occurred while waiting for the client to transition from this
    /// state.
    StateTimeout(State),
    /// Unexpected TCP disconnection.
    Disconnect,
    /// TCP connection timeout.
    TcpTimeout,
    /// A packet failed to decode.
    ///
    /// For example, this can occur when the variable byte encoding in the fixed
    /// header is incorrect.
    Decode,
    /// Protocol Error
    ///
    /// The packet encoding is correct, but an illegal value was used for
    /// one of the fields.
    Protocol,
    /// The client was unable to connect with the server.
    ///
    /// The next call to `process` will re-connect.
    ConnAck(ConnectReasonCode),
    /// Ran out of memory writing to the socket buffers.
    OutOfMemory,
    /// Tried to publish, subscribe, or unsubscribe when not connected.
    NotConnected,
    /// Alert from the TLS server.
    #[cfg(feature = "w5500-tls")]
    ServerAlert(w5500_tls::Alert),
    /// Alert from the TLS client.
    #[cfg(feature = "w5500-tls")]
    ClientAlert(w5500_tls::Alert),
    /// Errors from the [`Registers`] trait implementation.
    Other(E),
}

impl<E> Error<E> {
    fn map_w5500(e: w5500_hl::Error<E>) -> Error<E> {
        match e {
            HlError::OutOfMemory => Error::OutOfMemory,
            HlError::UnexpectedEof => Error::Decode,
            HlError::Other(e) => Error::Other(e),
            _ => unreachable!(),
        }
    }

    #[cfg(feature = "w5500-tls")]
    fn map_w5500_infallible(e: Error<core::convert::Infallible>) -> Error<E> {
        match e {
            Error::StateTimeout(to) => Error::StateTimeout(to),
            Error::Disconnect => Error::Disconnect,
            Error::TcpTimeout => Error::TcpTimeout,
            Error::Decode => Error::Decode,
            Error::Protocol => Error::Protocol,
            Error::ConnAck(reason) => Error::ConnAck(reason),
            Error::OutOfMemory => Error::OutOfMemory,
            Error::ServerAlert(alert) => Error::ServerAlert(alert),
            Error::ClientAlert(alert) => Error::ClientAlert(alert),
            Error::NotConnected => Error::NotConnected,
            Error::Other(_) => unreachable!(),
        }
    }
}

impl<E> From<E> for Error<E> {
    fn from(e: E) -> Self {
        Self::Other(e)
    }
}

/// length of the property length field
const PROPERTY_LEN_LEN: u16 = 1;

// length of the filter length field
const FILTER_LEN_LEN: u16 = 2;

// length of packet identifier field
const PACKET_ID_LEN: u32 = 2;

/// W5500 MQTT client.
///
/// # Topic Arguments
///
/// In the interest of minimal code size topic arguments are not validated,
/// you must adhere to the topic rules yourself, see
/// [Topic Names and Topic Filters].
/// The MQTT server will disconnect if an invalid topic is used.
///
/// [Topic Names and Topic Filters]: https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901241
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Client<'a> {
    sn: Sn,
    src_port: u16,
    server: SocketAddrV4,
    client_id: Option<ClientId<'a>>,
    /// State and Timeout tracker
    state_timeout: StateTimeout,
    /// Packet ID for subscribing
    pkt_id: u16,
}

impl<'a> Client<'a> {
    /// Create a new MQTT client.
    ///
    /// # Arguments
    ///
    /// * `sn` - The socket number to use for MQTT.
    /// * `src_port` - The MQTT source port, this is typically [`SRC_PORT`].
    /// * `server` - The IP address and port for the MQTT server.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_mqtt::{
    ///     ll::{
    ///         net::{Ipv4Addr, SocketAddrV4},
    ///         Sn,
    ///     },
    ///     Client, DST_PORT, SRC_PORT,
    /// };
    ///
    /// let client: Client = Client::new(
    ///     Sn::Sn2,
    ///     SRC_PORT,
    ///     SocketAddrV4::new(Ipv4Addr::new(192, 168, 5, 6), DST_PORT),
    /// );
    /// ```
    pub fn new(sn: Sn, src_port: u16, server: SocketAddrV4) -> Self {
        Self {
            sn,
            src_port,
            server,
            state_timeout: StateTimeout {
                state: State::Init,
                timeout: None,
            },
            client_id: None,
            pkt_id: 1,
        }
    }

    /// Set the MQTT client ID.
    ///
    /// This will only apply for new connections.
    /// Call this after [`new`], before calling [`process`].
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_mqtt::{
    ///     ll::{
    ///         net::{Ipv4Addr, SocketAddrV4},
    ///         Sn,
    ///     },
    ///     Client, ClientId, DST_PORT, SRC_PORT,
    /// };
    ///
    /// let mut client: Client = Client::new(
    ///     Sn::Sn2,
    ///     SRC_PORT,
    ///     SocketAddrV4::new(Ipv4Addr::new(192, 168, 5, 6), DST_PORT),
    /// );
    ///
    /// const CLIENT_ID: ClientId = ClientId::new_unwrapped("MYDEVICE");
    /// client.set_client_id(CLIENT_ID);
    /// ```
    ///
    /// [`new`]: Self::new
    /// [`process`]: Self::process
    pub fn set_client_id(&mut self, client_id: ClientId<'a>) {
        self.client_id = Some(client_id)
    }

    fn next_pkt_id(&mut self) -> u16 {
        self.pkt_id = self.pkt_id.checked_add(1).unwrap_or(1);
        self.pkt_id
    }

    /// Process the MQTT client.
    ///
    /// This should be called repeatedly until it returns:
    ///
    /// * `Err(_)` What to do upon errors is up to you.
    ///   Calling `process` again will re-initialize the client.
    /// * `Ok(Event::CallAfter(seconds))` Call this method again after the number
    ///   of seconds indicated.
    /// * `Ok(Event::None)` The client is idle; you can call [`subscribe`] and [`publish`].
    ///
    /// This should also be called when there is a pending socket interrupt.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use std::io::{ErrorKind as W5500Error};
    /// # use w5500_regsim::{W5500 as MyW5500};
    /// # fn spawn_at_this_many_seconds_in_the_future(s: u32) {}
    /// # fn monotonic_secs() -> u32 { 0 }
    /// use w5500_mqtt::{Client, Error, Event};
    ///
    /// fn my_rtos_task(client: &mut Client, w5500: &mut MyW5500) -> Result<(), Error<W5500Error>> {
    ///     loop {
    ///         match client.process(w5500, monotonic_secs()) {
    ///             Ok(Event::ConnAck) => {
    ///                 client.subscribe(w5500, "/demo/topic/#")?;
    ///             }
    ///             Ok(Event::CallAfter(secs)) => {
    ///                 spawn_at_this_many_seconds_in_the_future(secs);
    ///                 break;
    ///             }
    ///             Ok(Event::Publish(mut reader)) => {
    ///                 let mut topic_buf: [u8; 32] = [0; 32];
    ///                 let mut payload_buf: [u8; 32] = [0; 32];
    ///                 let topic_len: u16 = reader.read_topic(&mut topic_buf)?;
    ///                 let payload_len: u16 = reader.read_payload(&mut payload_buf)?;
    ///                 reader.done()?;
    ///
    ///                 // do something with the topic and payload
    ///             }
    ///             Ok(Event::SubAck(ack)) => {
    ///                 // this does not handle failed subscriptions
    ///                 log::info!("SubAck {:?}", ack)
    ///             }
    ///             Ok(Event::UnSubAck(ack)) => {
    ///                 log::info!("UnsubAck {:?}", ack)
    ///             }
    ///             Ok(Event::None) => break,
    ///             Err(e) => {
    ///                 log::error!("oh no, an error! {e:?}");
    ///                 // try again in a minute
    ///                 spawn_at_this_many_seconds_in_the_future(60);
    ///                 break;
    ///             }
    ///         }
    ///     }
    ///
    ///     Ok(())
    /// }
    /// # Ok::<(), w5500_mqtt::Error<std::io::ErrorKind>>(())
    /// ```
    ///
    /// [`subscribe`]: Self::subscribe
    /// [`publish`]: Self::publish
    #[allow(clippy::type_complexity)]
    pub fn process<'w, W5500: Registers>(
        &mut self,
        w5500: &'w mut W5500,
        monotonic_secs: u32,
    ) -> Result<Event<W5500::Error, TcpReader<'w, W5500>>, Error<W5500::Error>> {
        if self.state_timeout.state == State::Init {
            let call_after: u32 = self.tcp_connect(w5500, monotonic_secs)?;
            return Ok(Event::CallAfter(call_after));
        }

        let sn_ir: SocketInterrupt = w5500.sn_ir(self.sn)?;

        if sn_ir.any_raised() {
            w5500.set_sn_ir(self.sn, sn_ir)?;

            if sn_ir.discon_raised() {
                // TODO: try to get discon reason from server
                info!("DISCON interrupt");
                self.state_timeout.set_state(State::Init);
                return Err(Error::Disconnect);
            }
            if sn_ir.con_raised() {
                info!("CONN interrupt");
                let call_after: u32 = self.send_connect(w5500, monotonic_secs)?;
                return Ok(Event::CallAfter(call_after));
            }
            if sn_ir.timeout_raised() {
                info!("TIMEOUT interrupt");
                self.state_timeout.set_state(State::Init);
                return Err(Error::TcpTimeout);
            }
            if sn_ir.sendok_raised() {
                info!("SENDOK interrupt");
            }
            if sn_ir.recv_raised() {
                info!("RECV interrupt");
            }
        }

        match w5500.tcp_reader(self.sn) {
            Ok(reader) => match recv(reader, &mut self.state_timeout)? {
                Some(event) => return Ok(event),
                None => (),
            },
            Err(HlError::WouldBlock) => (),
            Err(HlError::Other(bus)) => return Err(Error::Other(bus)),
            Err(_) => unreachable!(),
        }

        if let Some(elapsed_secs) = self.state_timeout.timeout_elapsed_secs(monotonic_secs) {
            if elapsed_secs > TIMEOUT_SECS {
                info!(
                    "timeout waiting for state to transition from {:?}",
                    self.state_timeout.state
                );
                let ret = Err(Error::StateTimeout(self.state_timeout.state));
                self.state_timeout.set_state(State::Init);
                ret
            } else {
                let call_after: u32 = TIMEOUT_SECS.saturating_sub(elapsed_secs);
                Ok(Event::CallAfter(call_after))
            }
        } else {
            Ok(Event::None)
        }
    }

    /// Returns `true` if the MQTT client is connected.
    #[inline]
    pub fn is_connected(&self) -> bool {
        matches!(self.state_timeout.state, State::Ready)
    }

    fn connected<E>(&self) -> Result<(), Error<E>> {
        if self.is_connected() {
            Ok(())
        } else {
            Err(Error::NotConnected)
        }
    }

    /// Publish data to the MQTT broker.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # fn monotonic_secs() -> u32 { 0 }
    /// # let mut w5500 = w5500_regsim::W5500::default();
    /// use w5500_mqtt::{
    ///     ll::{
    ///         net::{Ipv4Addr, SocketAddrV4},
    ///         Sn,
    ///     },
    ///     Client, ClientId, Event, DST_PORT, SRC_PORT,
    /// };
    ///
    /// let mut client: Client = Client::new(
    ///     Sn::Sn2,
    ///     SRC_PORT,
    ///     SocketAddrV4::new(Ipv4Addr::new(192, 168, 5, 6), DST_PORT),
    /// );
    ///
    /// // wait for a connection or die trying
    /// while client.process(&mut w5500, monotonic_secs())? != Event::None {}
    ///
    /// client.publish(&mut w5500, "topic", b"data")?;
    /// # Ok::<(), w5500_mqtt::Error<std::io::ErrorKind>>(())
    /// ```
    ///
    /// [`is_connected`]: Self::is_connected
    pub fn publish<W5500: Registers>(
        &mut self,
        w5500: &mut W5500,
        topic: &str,
        payload: &[u8],
    ) -> Result<(), Error<W5500::Error>> {
        self.connected()?;
        let writer: TcpWriter<W5500> = w5500.tcp_writer(self.sn)?;
        send_publish(writer, topic, payload).map_err(Error::map_w5500)
    }

    /// Subscribe to a topic.
    ///
    /// # Return Value
    ///
    /// The return value is a `u16` packet identifier.
    /// This can be compared to `Event::SubAck` to determine when the
    /// subscription is active.
    ///
    /// The packet identifier is zero (invalid) when `filter` is empty.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # fn monotonic_secs() -> u32 { 0 }
    /// # let mut w5500 = w5500_regsim::W5500::default();
    /// use w5500_mqtt::{
    ///     ll::{
    ///         net::{Ipv4Addr, SocketAddrV4},
    ///         Sn,
    ///     },
    ///     Client, ClientId, Event, DST_PORT, SRC_PORT,
    /// };
    ///
    /// let mut client: Client = Client::new(
    ///     Sn::Sn2,
    ///     SRC_PORT,
    ///     SocketAddrV4::new(Ipv4Addr::new(192, 168, 5, 6), DST_PORT),
    /// );
    ///
    /// // wait for a connection or die trying
    /// while client.process(&mut w5500, monotonic_secs())? != Event::None {}
    ///
    /// client.subscribe(&mut w5500, "topic")?;
    /// # Ok::<(), w5500_mqtt::Error<std::io::ErrorKind>>(())
    /// ```
    ///
    /// [`is_connected`]: Self::is_connected
    pub fn subscribe<W5500: Registers>(
        &mut self,
        w5500: &mut W5500,
        filter: &str,
    ) -> Result<u16, Error<W5500::Error>> {
        self.connected()?;
        let writer: TcpWriter<W5500> = w5500.tcp_writer(self.sn)?;
        send_subscribe(writer, filter, self.next_pkt_id()).map_err(Error::map_w5500)
    }

    /// Unsubscribe from a topic.
    ///
    /// # Return Value
    ///
    /// The return value is a `u16` packet identifier.
    /// This can be compared to `Event::UnsubAck` to determine when the
    /// subscription has been deleted.
    ///
    /// The packet identifier is zero (invalid) when `filter` is empty.
    ///
    /// [Topic Names and Topic Filters]: https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901241
    /// [`is_connected`]: Self::is_connected
    pub fn unsubscribe<W5500: Registers>(
        &mut self,
        w5500: &mut W5500,
        filter: &str,
    ) -> Result<u16, Error<W5500::Error>> {
        self.connected()?;
        let writer: TcpWriter<W5500> = w5500.tcp_writer(self.sn)?;
        send_unsubscribe(writer, filter, self.next_pkt_id()).map_err(Error::map_w5500)
    }

    fn tcp_connect<W5500: Registers>(
        &mut self,
        w5500: &mut W5500,
        monotonic_secs: u32,
    ) -> Result<u32, W5500::Error> {
        let simr: u8 = w5500.simr()?;
        w5500.set_simr(self.sn.bitmask() | simr)?;
        const SN_IMR: SocketInterruptMask = SocketInterruptMask::DEFAULT.mask_sendok();
        w5500.set_sn_imr(self.sn, SN_IMR)?;
        w5500.tcp_connect(self.sn, self.src_port, &self.server)?;
        Ok(self
            .state_timeout
            .set_state_with_timeout(State::WaitConInt, monotonic_secs))
    }

    fn send_connect<W5500: Registers>(
        &mut self,
        w5500: &mut W5500,
        monotonic_secs: u32,
    ) -> Result<u32, Error<W5500::Error>> {
        // set recieve maximum property to RX socket buffer size
        let rx_max: u16 = w5500
            .sn_rxbuf_size(self.sn)?
            .unwrap_or_default()
            .size_in_bytes() as u16;

        let writer: TcpWriter<W5500> = w5500.tcp_writer(self.sn)?;
        send_connect(writer, &self.client_id, rx_max).map_err(Error::map_w5500)?;
        Ok(self
            .state_timeout
            .set_state_with_timeout(State::WaitConAck, monotonic_secs))
    }
}
