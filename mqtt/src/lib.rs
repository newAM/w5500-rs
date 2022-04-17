//! MQTT v5 client implementation for use with the W5500.
//!
//! # Limitations
//!
//! This is very basic at the moment, and will be expanded in the future.
//!
//! * Does not support password protected MQTT servers.
//! * Does not support TLS.
//! * Does not support unsubscribing.
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
//! // publish "quack" with a payload "oink"
//! client.publish(&mut w5500, "quack", b"oink")?;
//!
//! // subscribe to "moo"
//! client.subscribe(&mut w5500, "moo")?;
//! # Ok::<(), w5500_mqtt::Error<std::io::Error>>(())
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
//!
//! [w5500-hl]: https://crates.io/crates/w5500-hl
//! [`std::net`]: https://doc.rust-lang.org/std/net/index.html
//! [Wiznet W5500]: https://www.wiznet.io/product-item/w5500/
#![cfg_attr(docsrs, feature(doc_cfg), feature(doc_auto_cfg))]
#![cfg_attr(all(not(feature = "std"), not(test)), no_std)]
#![deny(unsafe_code)]
#![warn(missing_docs)]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

mod client_id;
mod connack;
mod data;
mod fixed_header;
mod properties;
mod subscribe;

pub use client_id::ClientId;
use connack::ConnectReasonCode;
use core::{cmp::min, mem::size_of};
use fixed_header::FixedHeader;
use hl::{
    ll::{net::SocketAddrV4, Registers, Sn, SocketInterrupt, SocketInterruptMask},
    Common, Error as HlError, Read, Seek, SeekFrom, Tcp, TcpReader, Writer,
};
use properties::Properties;
pub use subscribe::SubAckReasonCode;
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
    WaitConInt,
    /// CONNECT packet has been sent, waiting for a CONNACK.
    WaitConAck,
    /// CONNACK has been received, ready for action.
    Ready,
}

/// Duration in seconds to wait for the MQTT server to send a response.
const TIMEOUT_SECS: u32 = 10;

/// W5500 MQTT client.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Client<'a> {
    sn: Sn,
    port: u16,
    server: SocketAddrV4,
    client_id: Option<ClientId<'a>>,
    /// MQTT client state
    state: State,
    /// Timeout for MQTT server responses
    timeout: Option<u32>,
    /// Packet ID for subscribing
    pkt_id: u16,
}

fn write_variable_byte_integer<W5500: Registers>(
    writer: &mut Writer<W5500>,
    integer: u32,
) -> Result<(), Error<W5500::Error>> {
    let (buf, len): ([u8; 4], usize) = crate::data::encode_variable_byte_integer(integer);
    writer.write_all(&buf[..len]).map_err(map_write_all_err)
}

/// Reader for a published message on a subscribed topic.
///
/// This reads publish data directly from the socket buffer, avoiding the need
/// for an intermediate copy.
///
/// Created by [`Client::process`] when there is a pending message.
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PublishReader<'a, W5500: Registers> {
    reader: TcpReader<'a, W5500>,
    topic_len: u16,
    topic_idx: u16,
    payload_len: u16,
    payload_idx: u16,
}

impl<'a, W5500: Registers> PublishReader<'a, W5500> {
    /// Length of the topic in bytes.
    #[inline]
    pub fn topic_len(&self) -> u16 {
        self.topic_len
    }

    /// Length of the payload in bytes.
    #[inline]
    pub fn payload_len(&self) -> u16 {
        self.payload_len
    }

    /// Read the topic into `buf`, and return the number of bytes read.
    pub fn read_topic(&mut self, buf: &mut [u8]) -> Result<u16, Error<W5500::Error>> {
        self.reader
            .seek(SeekFrom::Start(self.topic_idx))
            .map_err(map_read_exact_err)?;
        let read_len: u16 = min(buf.len().try_into().unwrap_or(u16::MAX), self.topic_len);
        self.reader
            .read_exact(&mut buf[..read_len.into()])
            .map_err(map_read_exact_err)?;
        Ok(read_len)
    }

    /// Read the payload into `buf`, and return the number of bytes read.
    pub fn read_payload(&mut self, buf: &mut [u8]) -> Result<u16, Error<W5500::Error>> {
        self.reader
            .seek(SeekFrom::Start(self.payload_idx))
            .map_err(map_read_exact_err)?;
        let read_len: u16 = min(buf.len().try_into().unwrap_or(u16::MAX), self.payload_len);
        self.reader
            .read_exact(&mut buf[..read_len.into()])
            .map_err(map_read_exact_err)?;
        Ok(read_len)
    }

    /// Mark this message as read.
    ///
    /// If this is not called the message will be returned to the queue,
    /// available upon the next call to [`Client::process`].
    #[inline]
    pub fn done(self) -> Result<(), W5500::Error> {
        self.reader.done()?;
        Ok(())
    }
}

/// MQTT events.
///
/// These are events that need to be handled externally by your firmware,
/// such as a published message on a subscribed topic.
///
/// This is returned by [`Client::process`].
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Event<'a, W5500: Registers> {
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
    Publish(PublishReader<'a, W5500>),
    /// Subscribe Acknowledgment.
    SubAck {
        /// The inner value contains the packet identifier.
        /// This can be compared with the return value of [`Client::subscribe`] to
        /// determine which subscribe is being acknowledged.
        pkt_id: u16,
        /// SUBACK reason code.
        ///
        /// This should be checked to ensure the SUBSCRIBE was successful.
        code: SubAckReasonCode,
    },
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
    /// Errors from the [`Registers`] trait implementation.
    Other(E),
}

impl<E> From<E> for Error<E> {
    fn from(e: E) -> Self {
        Self::Other(e)
    }
}

// also maps seek errors when reading
fn map_read_exact_err<E>(e: w5500_hl::Error<E>) -> Error<E> {
    match e {
        HlError::UnexpectedEof => Error::Decode,
        HlError::Other(e) => Error::Other(e),
        _ => unreachable!(),
    }
}

fn map_write_all_err<E>(e: w5500_hl::Error<E>) -> Error<E> {
    match e {
        HlError::OutOfMemory => Error::OutOfMemory,
        HlError::Other(e) => Error::Other(e),
        _ => unreachable!(),
    }
}

/// length of the property length field
const PROPERTY_LEN_LEN: u16 = 1;

impl<'a> Client<'a> {
    /// Create a new MQTT client.
    ///
    /// # Arguments
    ///
    /// * `sn` - The socket number to use for MQTT.
    /// * `port` - The MQTT source port, this is typically [`SRC_PORT`].
    /// * `server` - The IP address and port for the MQTT server.
    /// * `subscribe_filters` - A list of topic filters to subscribe to.
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
    pub fn new(sn: Sn, port: u16, server: SocketAddrV4) -> Self {
        Self {
            sn,
            port,
            server,
            state: State::Init,
            client_id: None,
            timeout: None,
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

    /// Process the MQTT client.
    ///
    /// This should be called repeatedly until it returns:
    ///
    /// * `Err(_)` What to do upon errors is up to you.
    ///   Calling `process` again will re-initialize the client.
    /// * `Ok(Event::CallAfter(seconds))` Call this method again after the number
    ///   of seconds indicated.
    /// * `Ok(Event::None)` The client is idle; you can  call [`subscribe`] and [`publish`].
    ///
    /// This should also be called when there is a pending socket interrupt.
    ///
    /// [`subscribe`]: Self::subscribe
    /// [`publish`]: Self::publish
    pub fn process<'w, W5500: Registers>(
        &mut self,
        w5500: &'w mut W5500,
        monotonic_secs: u32,
    ) -> Result<Event<'w, W5500>, Error<W5500::Error>> {
        if self.state == State::Init {
            let call_after: u32 = self.tcp_connect(w5500, monotonic_secs)?;
            return Ok(Event::CallAfter(call_after));
        }

        let sn_ir: SocketInterrupt = w5500.sn_ir(self.sn)?;

        if sn_ir.any_raised() {
            w5500.set_sn_ir(self.sn, sn_ir)?;

            if sn_ir.discon_raised() {
                // TODO: try to get discon reason from server
                info!("DISCON interrupt");
                self.set_state(State::Init);
                return Err(Error::Disconnect);
            }
            if sn_ir.con_raised() {
                info!("CONN interrupt");
                self.set_state(State::Init);
                let call_after: u32 = self.send_connect(w5500, monotonic_secs)?;
                return Ok(Event::CallAfter(call_after));
            }
            if sn_ir.timeout_raised() {
                info!("TIMEOUT interrupt");
                self.set_state(State::Init);
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
            Ok(reader) => match self.recv(reader)? {
                Some(event) => return Ok(event),
                None => (),
            },
            Err(HlError::WouldBlock) => (),
            Err(HlError::Other(bus)) => return Err(Error::Other(bus)),
            Err(_) => unreachable!(),
        }

        if let Some(elapsed_secs) = self.timeout_elapsed_secs(monotonic_secs) {
            if elapsed_secs > TIMEOUT_SECS {
                info!(
                    "timeout waiting for state to transition from {:?}",
                    self.state
                );
                let ret = Err(Error::StateTimeout(self.state));
                self.set_state(State::Init);
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
        matches!(self.state, State::Ready)
    }

    /// Publish data to the MQTT broker.
    ///
    /// You can only subscribe when the client [`is_connected`].
    /// If you are not connected and write data this function will return
    /// `Ok(())`, and the connection process will restart.
    ///
    /// In the interest of minimal code size the topic is not validated,
    /// you must adhere to the topic rules yourself, see
    /// [Topic Names and Topic Filters].
    /// The MQTT server will disconnect if an invalid topic is used.
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
    /// # Ok::<(), w5500_mqtt::Error<std::io::Error>>(())
    /// ```
    ///
    /// [Topic Names and Topic Filters]: https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901241
    /// [`is_connected`]: Self::is_connected
    pub fn publish<W5500: Registers>(
        &mut self,
        w5500: &mut W5500,
        topic: &str,
        payload: &[u8],
    ) -> Result<(), Error<W5500::Error>> {
        let topic_len: u16 = topic.len().try_into().unwrap_or(u16::MAX);
        let payload_len: u16 = payload.len().try_into().unwrap_or(u16::MAX);

        // length of the topic length field
        const TOPIC_LEN_LEN: u32 = size_of::<u16>() as u32;
        // length of the property length field
        const PROPERTY_LEN: u32 = size_of::<u8>() as u32;
        let remaining_len: u32 =
            TOPIC_LEN_LEN + u32::from(topic_len) + PROPERTY_LEN + u32::from(payload_len);

        let mut writer: Writer<W5500> = w5500.writer(self.sn)?;
        writer
            .write_all(&[
                // control packet type
                // flags are all 0
                // dup=0, non-duplicate
                // qos=0, at most once delivery
                // retain=0, do not retain this message
                (CtrlPkt::PUBLISH as u8) << 4,
            ])
            .map_err(map_write_all_err)?;
        write_variable_byte_integer(&mut writer, remaining_len)?;
        writer
            .write_all(&topic_len.to_be_bytes())
            .map_err(map_write_all_err)?;
        writer
            .write_all(&topic.as_bytes()[..topic_len.into()])
            .map_err(map_write_all_err)?;
        writer.write_all(&[0]).map_err(map_write_all_err)?; // property length
        writer
            .write_all(&payload[..payload_len.into()])
            .map_err(map_write_all_err)?;
        writer.send()?;
        Ok(())
    }

    /// Subscribe to a topic.
    ///
    /// You can only subscribe when the client [`is_connected`].
    /// If you are not connected and write data this function will return
    /// `Ok(())`, and the connection process will restart.
    ///
    /// In the interest of minimal code size the topic is not validated,
    /// you must adhere to the topic rules yourself, see
    /// [Topic Names and Topic Filters].
    /// The MQTT server will disconnect if an invalid topic is used.
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
    /// # Ok::<(), w5500_mqtt::Error<std::io::Error>>(())
    /// ```
    ///
    /// [Topic Names and Topic Filters]: https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901241
    /// [`is_connected`]: Self::is_connected
    pub fn subscribe<W5500: Registers>(
        &mut self,
        w5500: &mut W5500,
        filter: &str,
    ) -> Result<u16, Error<W5500::Error>> {
        if filter.is_empty() {
            Ok(0)
        } else {
            // length of the filter length field
            const FILTER_LEN_LEN: u16 = 2;
            const SUBSCRIPTION_OPTIONS_LEN: u16 = 1;

            let filter_len: u16 = (filter.len() as u16) + FILTER_LEN_LEN + SUBSCRIPTION_OPTIONS_LEN;

            // length of packet identifier field
            const PACKET_ID_LEN: u32 = 2;

            let remaining_len: u32 =
                PACKET_ID_LEN + u32::from(PROPERTY_LEN_LEN) + u32::from(filter_len);

            let mut writer: Writer<W5500> = w5500.writer(self.sn)?;
            writer
                .write_all(&[(CtrlPkt::SUBSCRIBE as u8) << 4 | 0b0010])
                .map_err(map_write_all_err)?;
            write_variable_byte_integer(&mut writer, remaining_len)?;
            let pkt_id: u16 = self.next_pkt_id();
            writer
                .write_all(&[
                    // packet identifier
                    (pkt_id >> 8) as u8,
                    pkt_id as u8,
                    // property length
                    0,
                ])
                .map_err(map_write_all_err)?;

            writer
                .write_all(
                    u16::try_from(filter.len())
                        .unwrap_or(u16::MAX)
                        .to_be_bytes()
                        .as_ref(),
                )
                .map_err(map_write_all_err)?;
            writer
                .write_all(filter.as_bytes())
                .map_err(map_write_all_err)?;
            // subscription options flags
            // 00 => reserved
            // 10 => retain handling: do not set messages at subscribtion time
            // 0 => retain as published: all messages have the retain flag cleared
            // 1 => no local option: do not send messages published by this client
            // 00 => QoS 0: at most once delivery
            writer.write_all(&[0b00100100]).map_err(map_write_all_err)?;

            writer.send()?;

            Ok(pkt_id)
        }
    }

    fn tcp_connect<W5500: Registers>(
        &mut self,
        w5500: &mut W5500,
        monotonic_secs: u32,
    ) -> Result<u32, W5500::Error> {
        const SN_IMR: SocketInterruptMask = SocketInterruptMask::DEFAULT.mask_sendok();
        w5500.set_sn_imr(self.sn, SN_IMR)?;
        w5500.tcp_connect(self.sn, self.port, &self.server)?;
        Ok(self.set_state_with_timeout(State::WaitConInt, monotonic_secs))
    }

    fn send_connect<W5500: Registers>(
        &mut self,
        w5500: &mut W5500,
        monotonic_secs: u32,
    ) -> Result<u32, Error<W5500::Error>> {
        const KEEP_ALIVE: u16 = 15 * 60;

        let client_id_len: u8 = self.client_id.map(|id| id.len()).unwrap_or(0);

        // set recieve maximum property to RX socket buffer size
        let rx_max: u16 = w5500
            .sn_rxbuf_size(self.sn)?
            .unwrap_or_default()
            .size_in_bytes() as u16;

        let mut writer: Writer<W5500> = w5500.writer(self.sn)?;
        #[rustfmt::skip]
        writer.write_all(&[
            // control packet type
            (CtrlPkt::CONNECT as u8) << 4,
            // remaining length
            18 + client_id_len,
            // protocol name length
            0, 4,
            // protocol name
            b'M', b'Q', b'T', b'T',
            // protocol version
            5,
            // flags, clean start is set
            0b00000010,
            // keepalive
            (KEEP_ALIVE >> 8) as u8, KEEP_ALIVE as u8,
            // properties length
            5,
            // recieve maximum property
            (Properties::MaxPktSize as u8), 0, 0, (rx_max >> 8) as u8, rx_max as u8,
            // client ID length
            0, client_id_len,
        ]).map_err(map_write_all_err)?;
        if let Some(client_id) = self.client_id {
            writer
                .write_all(client_id.as_bytes())
                .map_err(map_write_all_err)?;
        }
        writer.send()?;
        Ok(self.set_state_with_timeout(State::WaitConAck, monotonic_secs))
    }

    fn recv<'w, W5500: Registers>(
        &mut self,
        mut reader: TcpReader<'w, W5500>,
    ) -> Result<Option<Event<'w, W5500>>, Error<W5500::Error>> {
        let mut buf: [u8; 5] = [0; 5];
        let n: u16 = reader.read(&mut buf)?;

        let header: FixedHeader = match FixedHeader::deser(&buf[..n.into()]) {
            Some(header) => header,
            None => {
                error!("unable to deserialize fixed header");
                self.set_state(State::Init);
                return Err(Error::Decode);
            }
        };

        // seek to end of fixed header
        reader
            .seek(SeekFrom::Start(header.len.into()))
            .map_err(map_read_exact_err)?;

        debug!("recv {:?} len {}", header.ctrl_pkt, header.remaining_len);

        match header.ctrl_pkt {
            CtrlPkt::RESERVED => {
                error!("Malformed packet: control packet type is reserved");
                self.set_state(State::Init);
                Err(Error::Decode)
            }
            CtrlPkt::CONNACK => {
                if self.state != State::WaitConAck {
                    error!("unexpected CONNACK in state {:?}", self.state);
                    self.set_state(State::Init);
                    return Err(Error::Protocol);
                }
                let mut buf: [u8; 2] = [0; 2];
                reader.read_exact(&mut buf).map_err(map_read_exact_err)?;
                reader
                    .seek(SeekFrom::Start(2 + header.remaining_len))
                    .map_err(map_read_exact_err)?;
                reader.done()?;

                match ConnectReasonCode::try_from(buf[1]) {
                    Err(0) => {
                        info!("Sucessfully connected");
                        self.set_state(State::Ready);
                        Ok(Some(Event::None))
                    }
                    Ok(code) => {
                        warn!("Unable to connect: {:?}", code);
                        self.set_state(State::Init);
                        Err(Error::ConnAck(code))
                    }
                    Err(e) => {
                        error!("invalid connnect reason code {:?}", e);
                        self.set_state(State::Init);
                        Err(Error::Protocol)
                    }
                }
            }
            CtrlPkt::SUBACK => {
                let mut buf: [u8; 3] = [0; 3];
                let n: u16 = reader.read(&mut buf)?;
                if n != 3 {
                    return Err(Error::Decode);
                }

                let (pkt_id, property_len): (&[u8], &[u8]) = buf.split_at(2);
                let pkt_id: u16 = u16::from_be_bytes(pkt_id.try_into().unwrap());
                let property_len: u8 = property_len[0];

                if property_len != 0 {
                    warn!("ignoring SUBACK properties");
                    reader
                        .seek(SeekFrom::Current(property_len.into()))
                        .map_err(map_read_exact_err)?;
                }

                let mut payload: [u8; 1] = [0];
                reader
                    .read_exact(&mut payload)
                    .map_err(map_read_exact_err)?;
                let code: SubAckReasonCode = match SubAckReasonCode::try_from(payload[0]) {
                    Ok(code) => code,
                    Err(e) => {
                        error!("invalid SUBACK reason code value: {}", e);
                        self.set_state(State::Init);
                        return Err(Error::Protocol);
                    }
                };

                reader.done()?;
                Ok(Some(Event::SubAck { pkt_id, code }))
            }
            CtrlPkt::PUBLISH => {
                const TOPIC_LEN_LEN: u16 = 2;
                let mut topic_len: [u8; 2] = [0; 2];
                reader
                    .read_exact(&mut topic_len)
                    .map_err(map_read_exact_err)?;
                let topic_len: u16 = u16::from_be_bytes(topic_len);
                let topic_idx: u16 = reader.stream_position();
                reader
                    .seek(SeekFrom::Current(topic_len.try_into().unwrap_or(i16::MAX)))
                    .map_err(map_read_exact_err)?;

                let mut property_len: [u8; 1] = [0];
                reader
                    .read_exact(&mut property_len)
                    .map_err(map_read_exact_err)?;
                let property_len: u8 = property_len[0];
                if property_len != 0 {
                    warn!("ignoring PUBLISH properties");
                    reader
                        .seek(SeekFrom::Current(property_len.into()))
                        .map_err(map_read_exact_err)?;
                }

                let payload_len: u16 = header
                    .remaining_len
                    .saturating_sub(topic_len)
                    .saturating_sub(TOPIC_LEN_LEN)
                    .saturating_sub(PROPERTY_LEN_LEN)
                    .saturating_sub(u16::from(property_len));
                let payload_idx: u16 = reader.stream_position();

                Ok(Some(Event::Publish(PublishReader {
                    reader,
                    topic_len,
                    topic_idx,
                    payload_len,
                    payload_idx,
                })))
            }
            x => {
                warn!("Unhandled control packet: {:?}", x);
                reader
                    .seek(SeekFrom::Current(
                        header.remaining_len.try_into().unwrap_or(i16::MAX),
                    ))
                    .map_err(map_read_exact_err)?;
                reader.done()?;
                Ok(None)
            }
        }
    }
}
