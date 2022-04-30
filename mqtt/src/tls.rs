//! MQTT over TLS
//!
//! Please read the [`w5500-tls`] README for a list of limitations.
//!
//! # Server Setup
//!
//! The [mosquitto] broker does not support TLS v1.3 with pre-shared keys [[1]].
//! You can use a proxy on-top of [mosquitto] to provide a TLS v1.3 socket that
//! accepts pre-shared keys.
//!
//! For example, with [stunnel]:
//!
//! ```text
//! [PSK server]
//! accept = 8883
//! connect = 1883
//! ciphers = PSK
//! PSKsecrets = /run/secrets/mosquitto-psk-file
//! ```
//!
//! Where `/run/secrets/mosquitto-psk-file` contains lines in the format of
//! `idenitiy:hexpsk`:
//!
//! ```text
//! test:2f42ace2b6be1681b3d2fcdd4bb57b4ffe3484ee77fdaa8e216e3272cd78259d
//! ```
//!
//! You can generate a random 256-bit hex string with `openssl`:
//!
//! ```console
//! $ openssl rand -hex 32
//! 2f42ace2b6be1681b3d2fcdd4bb57b4ffe3484ee77fdaa8e216e3272cd78259d
//! ```
//!
//! Then in your firmware you create a matching PSK and identity.
//!
//! ```
//! const IDENTITY: &[u8] = b"test";
//! const KEY: [u8; 32] = [
//!     0x2f, 0x42, 0xac, 0xe2, 0xb6, 0xbe, 0x16, 0x81, 0xb3, 0xd2, 0xfc, 0xdd, 0x4b, 0xb5, 0x7b,
//!     0x4f, 0xfe, 0x34, 0x84, 0xee, 0x77, 0xfd, 0xaa, 0x8e, 0x21, 0x6e, 0x32, 0x72, 0xcd, 0x78,
//!     0x25, 0x9d,
//! ];
//! ```
//!
//! `const` is just for an example, typically you will store these secrets
//! in a secure non-volatile memory to avoid using private keys in your
//! version control, and to give each device a unique key while using the same
//! firmware.
//!
//! [1]: https://github.com/eclipse/mosquitto/blob/4be56239e99ab4ef47c5ad6089f4a4e7f8ef97f8/ChangeLog.txt#L67
//! [mosquitto]: https://github.com/eclipse/mosquitto
//! [stunnel]: https://www.stunnel.org/
//! [`w5500-tls`]: https://github.com/newAM/w5500-rs/blob/main/tls/README.md

use crate::{
    connect::send_connect,
    hl::{
        ll::{net::SocketAddrV4, Registers, Sn},
        Error as HlError, Hostname,
    },
    publish::send_publish,
    recv::recv,
    subscribe::{send_subscribe, send_unsubscribe},
    ClientId, Error, Event, State, StateTimeout, TIMEOUT_SECS,
};
use core::convert::Infallible;
use w5500_tls::{
    rand_core::{CryptoRng, RngCore},
    Client as TlsClient, Error as TlsError, Event as TlsEvent, TlsReader, TlsWriter,
};

/// Default MQTT TLS destination port.
pub const DST_PORT: u16 = 8883;

fn map_tls_writer_err<E>(e: w5500_tls::Error) -> Error<E> {
    match e {
        TlsError::UnexpectedDisconnect | TlsError::TcpTimeout | TlsError::StateTimeout(_) => {
            unreachable!()
        }
        TlsError::Server(alert) => Error::ServerAlert(alert),
        TlsError::Client(alert) => Error::ClientAlert(alert),
        TlsError::NotConnected => Error::NotConnected,
    }
}

/// W5500 MQTT client over TLS.
///
/// The methods are nearly identical to [`crate::Client`], see [`crate::Client`]
/// for additional documentation and examples.
pub struct Client<'id, 'hn, 'psk, 'b, const N: usize> {
    tls: TlsClient<'hn, 'psk, 'b, N>,
    client_id: Option<ClientId<'id>>,
    /// State and Timeout tracker
    state_timeout: StateTimeout,
    /// Packet ID for subscribing
    pkt_id: u16,
}

impl<'id, 'hn, 'psk, 'b, const N: usize> Client<'id, 'hn, 'psk, 'b, N> {
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
    /// # fn load_identity_from_memory() -> &'static [u8] { &[] }
    /// # fn load_key_from_memory() -> &'static [u8] { &[] }
    /// use w5500_mqtt::{
    ///     hl::Hostname,
    ///     ll::{
    ///         net::{Ipv4Addr, SocketAddrV4},
    ///         Sn,
    ///     },
    ///     tls::{Client, DST_PORT},
    ///     SRC_PORT,
    /// };
    ///
    /// static mut RXBUF: [u8; 2048] = [0; 2048];
    ///
    /// let client: Client<2048> = Client::new(
    ///     Sn::Sn2,
    ///     SRC_PORT,
    ///     Hostname::new_unwrapped("mqtt.local"),
    ///     SocketAddrV4::new(Ipv4Addr::new(192, 168, 5, 6), DST_PORT),
    ///     load_identity_from_memory(),
    ///     &load_key_from_memory(),
    ///     unsafe { &mut RXBUF },
    /// );
    /// ```
    ///
    /// [`SRC_PORT`]: crate::SRC_PORT
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
            tls: TlsClient::new(sn, src_port, hostname, dst, identity, psk, rx),
            state_timeout: StateTimeout {
                state: State::Init,
                timeout: None,
            },
            client_id: None,

            pkt_id: 1,
        }
    }

    /// Set the MQTT client ID.
    pub fn set_client_id(&mut self, client_id: ClientId<'id>) {
        self.client_id = Some(client_id)
    }

    fn next_pkt_id(&mut self) -> u16 {
        self.pkt_id = self.pkt_id.checked_add(1).unwrap_or(1);
        self.pkt_id
    }

    /// Process the MQTT client.
    pub fn process<'w, 'ptr, W5500: Registers, R: RngCore + CryptoRng>(
        &'ptr mut self,
        w5500: &'w mut W5500,
        rng: &mut R,
        monotonic_secs: u32,
    ) -> Result<Event<Infallible, TlsReader<'b, 'ptr>>, Error<W5500::Error>> {
        loop {
            match self.tls.process(w5500, rng, monotonic_secs) {
                Err(TlsError::Server(alert)) => return Err(Error::ServerAlert(alert)),
                Err(TlsError::Client(alert)) => return Err(Error::ClientAlert(alert)),
                Err(TlsError::UnexpectedDisconnect) => return Err(Error::Disconnect),
                Err(TlsError::TcpTimeout) => return Err(Error::TcpTimeout),
                Err(TlsError::StateTimeout(tls_state)) => {
                    info!("TLS state timeout {:?}", tls_state);
                    return Err(Error::StateTimeout(State::WaitConAck));
                }
                Err(TlsError::NotConnected) => unreachable!(),
                Ok(TlsEvent::CallAfter(after)) => return Ok(Event::CallAfter(after)),
                Ok(TlsEvent::ApplicationData) => break,
                Ok(TlsEvent::HandshakeFinished) => {
                    let call_after: u32 = self.send_connect(w5500, monotonic_secs)?;
                    return Ok(Event::CallAfter(call_after));
                }
                Ok(TlsEvent::Disconnect) => return Err(Error::Disconnect),
                Ok(TlsEvent::None) => break,
            }
        }

        match self.tls.reader() {
            Ok(reader) => {
                match recv(reader, &mut self.state_timeout).map_err(Error::map_w5500_infallible)? {
                    Some(event) => return Ok(event),
                    None => (),
                }
            }
            Err(HlError::WouldBlock) => (),
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

    /// Publish data to the MQTT broker.
    pub fn publish<W5500: Registers>(
        &mut self,
        w5500: &mut W5500,
        topic: &str,
        payload: &[u8],
    ) -> Result<(), Error<W5500::Error>> {
        let writer: TlsWriter<W5500> = self.tls.writer(w5500).map_err(map_tls_writer_err)?;
        send_publish(writer, topic, payload).map_err(Error::map_w5500)
    }

    /// Subscribe to a topic.
    pub fn subscribe<W5500: Registers>(
        &mut self,
        w5500: &mut W5500,
        filter: &str,
    ) -> Result<u16, Error<W5500::Error>> {
        let pkt_id: u16 = self.next_pkt_id();
        let writer: TlsWriter<W5500> = self.tls.writer(w5500).map_err(map_tls_writer_err)?;
        send_subscribe(writer, filter, pkt_id).map_err(Error::map_w5500)
    }

    /// Unsubscribe from a topic.
    pub fn unsubscribe<W5500: Registers>(
        &mut self,
        w5500: &mut W5500,
        filter: &str,
    ) -> Result<u16, Error<W5500::Error>> {
        let pkt_id: u16 = self.next_pkt_id();
        let writer: TlsWriter<W5500> = self.tls.writer(w5500).map_err(map_tls_writer_err)?;
        send_unsubscribe(writer, filter, pkt_id).map_err(Error::map_w5500)
    }

    fn send_connect<W5500: Registers>(
        &mut self,
        w5500: &mut W5500,
        monotonic_secs: u32,
    ) -> Result<u32, Error<W5500::Error>> {
        // set recieve maximum property to RX socket buffer size, minus TLS
        // overhead
        const TLS_RECORD_HEADER_LEN: u16 = 5;
        const TLS_TAG_LEN: u16 = 16;
        const TLS_CONTENT_TYPE_LEN: u16 = 1;
        const TLS_OVERHEAD: u16 = TLS_RECORD_HEADER_LEN + TLS_TAG_LEN + TLS_CONTENT_TYPE_LEN;

        let rx_max: u16 = (N as u16) - TLS_OVERHEAD;
        let writer: TlsWriter<W5500> = self.tls.writer(w5500).map_err(map_tls_writer_err)?;
        send_connect(writer, &self.client_id, rx_max).map_err(Error::map_w5500)?;
        Ok(self
            .state_timeout
            .set_state_with_timeout(State::WaitConAck, monotonic_secs))
    }
}
