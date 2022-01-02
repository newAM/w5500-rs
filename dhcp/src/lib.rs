//! Simple DHCP client for the [Wiznet W5500] SPI internet offload chip.
//!
//! # Warning
//!
//! Please proceed with caution, and review the code before use in a production
//! environment.
//!
//! This code was developed for one-off hobby projects.
//! It works for my network, but it likely has bugs.
//!
//! ## Limitations
//!
//! * No support for rebinding
//! * No support for renewing
//!
//! # Feature Flags
//!
//! All features are disabled by default.
//!
//! * `embedded-hal`: Passthrough to [w5500-hl].
//! * `std`: Passthrough to [w5500-hl].
//! * `defmt`: Enable logging with `defmt`.  Mutually exclusive with `log`.
//!   Also a passthrough to [w5500-hl].
//! * `log`: Enable logging with `log`.  Mutually exclusive with `defmt`.
//!
//! # Related Crates
//!
//! * [w5500-ll] - Low level W5500 register accessors.
//! * [w5500-hl] - Higher level socket operations.
//! * [w5500-regsim] - Register simulation using [`std::net`].
//!
//! [`std::net`]: https://doc.rust-lang.org/std/net/index.html
//! [w5500-hl]: https://github.com/newAM/w5500-hl-rs
//! [w5500-ll]: https://github.com/newAM/w5500-ll-rs
//! [w5500-regsim]: https://github.com/newAM/w5500-regsim-rs
//! [Wiznet W5500]: https://www.wiznet.io/product-item/w5500/
#![cfg_attr(docsrs, feature(doc_cfg), feature(doc_auto_cfg))]
#![cfg_attr(all(not(feature = "std"), not(test)), no_std)]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

#[cfg(feature = "defmt")]
use dfmt as defmt;

mod pkt;
mod rand;

pub use w5500_hl as hl;
pub use w5500_hl::ll;

use pkt::{MsgType, PktDe, PktSer};
use w5500_hl::{
    ll::{
        net::{Ipv4Addr, SocketAddrV4},
        Registers, Sn,
    },
    nb::{self, Error::WouldBlock},
    net::Eui48Addr,
    Udp,
};

/// DHCP destination port.
pub const DHCP_DESTINATION_PORT: u16 = 67;
/// DHCP source port.
pub const DHCP_SOURCE_PORT: u16 = 68;

const DHCP_BROADCAST: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::BROADCAST, DHCP_DESTINATION_PORT);

/// DHCP client states.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[allow(missing_docs)]
pub enum State {
    Init,
    Selecting,
    Requesting,
    InitReboot,
    Rebooting,
    Bound,
    Renewing,
    Rebinding,
}

/// DHCP client storage.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Dhcp {
    /// Socket to use for the DHCP client.
    sn: Sn,
    /// DHCP client state
    state: State,
    /// Timeout for DHCP server responses
    timeout: Option<u32>,
    /// Renewal duration.
    t1: u32,
    /// Rebinding duration.
    t2: u32,
    /// Lease duration.
    lease: u32,
    /// Time that the lease was obtained.
    lease_monotonic_secs: u32,
    /// Last XID
    xid: u32,
    /// XID generator
    rand: rand::Rand,
    /// Hardware (EUI-48 MAC) address
    mac: Eui48Addr,
    /// IP address
    ip: Ipv4Addr,
    /// Client hostname
    hostname: &'static str,
}

impl Dhcp {
    /// Create a new DHCP client storage structure.
    ///
    /// The DHCP client can be reset by re-creating this structure.
    ///
    /// # Example
    ///
    /// ```
    /// use rand_core::RngCore;
    /// use w5500_dhcp::{
    ///     ll::{net::Eui48Addr, Sn},
    ///     Dhcp,
    /// };
    /// # let mut rng = rand_core::OsRng;
    ///
    /// const DHCP_SN: Sn = Sn::Sn0;
    /// // locally administered MAC address
    /// const MAC_ADDRESS: Eui48Addr = Eui48Addr::new(0x02, 0x00, 0x11, 0x22, 0x33, 0x44);
    /// const HOSTNAME: &str = "demo";
    /// let seed: u64 = rng.next_u64();
    ///
    /// let dhcp: Dhcp = Dhcp::new(DHCP_SN, seed, MAC_ADDRESS, HOSTNAME);
    /// ```
    pub fn new(sn: Sn, seed: u64, mac: Eui48Addr, hostname: &'static str) -> Self {
        let mut rand: rand::Rand = rand::Rand::new(seed);

        Self {
            sn,
            state: State::Init,
            timeout: None,
            t1: 0,
            t2: 0,
            lease: 0,
            lease_monotonic_secs: 0,
            xid: rand.next_u32(),
            rand,
            mac,
            ip: Ipv4Addr::UNSPECIFIED,
            hostname,
        }
    }

    /// Returns `true` if the DHCP client is bound.
    ///
    /// # Example
    ///
    /// ```
    /// use rand_core::RngCore;
    /// use w5500_dhcp::{
    ///     ll::{net::Eui48Addr, Sn},
    ///     Dhcp,
    /// };
    /// # let mut rng = rand_core::OsRng;
    ///
    /// let dhcp: Dhcp = Dhcp::new(
    ///     Sn::Sn0,
    ///     rng.next_u64(),
    ///     Eui48Addr::new(0x02, 0x00, 0x11, 0x22, 0x33, 0x44),
    ///     "demo",
    /// );
    /// assert_eq!(dhcp.is_bound(), false);
    /// ```
    #[inline]
    pub fn is_bound(&self) -> bool {
        self.state == State::Bound
    }
}

/// DHCP client errors.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error<W5500> {
    /// Error from the W5500.
    W5500(W5500),
    /// DHCP buffer is too small.
    ///
    /// When this error is returned the state of the W5500 has not changed.
    /// The function should be called again with a larger buffer.
    BufOverflow,
    /// W5500 TX buffer is too small.
    ///
    /// When this error is returned the state of the W5500 may have changed;
    /// but no data is transmitted.
    /// The function should be called again after adjusting the W5500 TX buffer
    /// size.
    TxOverflow,
    /// A requested DHCP option is missing.
    ///
    /// W5500 state was modified, reset the DHCP server and re-initialize.
    MissingOption,
}

/// DHCP client.
///
/// # Hardware Requirements
///
/// * The W5500 INT pin must be connected to the micronctroller.
///
/// # External Setup
///
/// 1. The W5500 INT pin must be configured for a falling edge trigger.
/// 2. RECV interrupts for the DHCP socket must be enabled with `set_simr`.
///
/// # System Time
///
/// You must supply a monotonic `u32` that counts the number of seconds since
/// system boot to the [`poll`] and [`on_recv_interrupt`] methods.
///
/// This is required for timeouts, and tracking the DHCP lease timers.
///
/// # Usage
///
/// Call the [`poll`] method approximately every second.
/// [`poll`] handles DHCP timer expiry, and timeouts if the DHCP
/// server fails to respond to DHCPDISCOVER or DHCPREQUEST.
///
/// Call [`on_recv_interrupt`] when there is a RECV interrupt on the DHCP
/// socket.  This will handle packet RX from the DHCP server.
///
/// [`poll`]: Client::poll
/// [`on_recv_interrupt`]: Client::on_recv_interrupt
#[derive(Debug)]
pub struct Client<'a, W5500> {
    w5500: &'a mut W5500,
    dhcp: &'a mut Dhcp,
    buf: &'a mut [u8],
}

impl<'a, W5500, E> Client<'a, W5500>
where
    W5500: Udp<Error = E> + Registers<Error = E>,
{
    /// Create a new DHCP client.
    ///
    /// This is intended to be called before before calling
    /// [`on_recv_interrupt`] or [`poll`].
    ///
    /// You will to use a large buffer for the DHCP client, typically 800 bytes
    /// is sufficient.
    ///
    /// [`poll`]: Client::poll
    /// [`on_recv_interrupt`]: Client::on_recv_interrupt
    #[inline]
    pub fn new(w5500: &'a mut W5500, dhcp: &'a mut Dhcp, buf: &'a mut [u8]) -> Self {
        Self { w5500, dhcp, buf }
    }

    /// Handle a RECV interrupt on the DHCP socket.
    ///
    /// This will **NOT** clear the socket interrupt.
    pub fn on_recv_interrupt(&mut self, monotonic_secs: u32) -> Result<(), Error<E>> {
        let state: State = self.dhcp.state;

        if let Some(pkt) = self.recv()? {
            match state {
                State::Selecting => {
                    self.dhcp.ip = unwrap!(pkt.yiaddr());
                    self.request(monotonic_secs)?;
                }
                State::Requesting | State::Renewing | State::Rebinding => {
                    match pkt.msg_type().ok_or(Error::MissingOption)? {
                        MsgType::Ack => {
                            let subnet_mask: Ipv4Addr =
                                pkt.subnet_mask().ok_or(Error::MissingOption)?;
                            info!("subnet_mask: {}", subnet_mask);
                            let gateway: Ipv4Addr =
                                pkt.dhcp_server().ok_or(Error::MissingOption)?;
                            info!("gateway: {}", gateway);
                            let renewal_time: u32 =
                                pkt.renewal_time().ok_or(Error::MissingOption)?;
                            info!("renewal_time: {}", renewal_time);
                            let rebinding_time: u32 =
                                pkt.rebinding_time().ok_or(Error::MissingOption)?;
                            info!("rebinding_time: {}", rebinding_time);
                            let lease_time: u32 = pkt.lease_time().ok_or(Error::MissingOption)?;
                            info!("lease_time: {}", lease_time);

                            self.dhcp.t1 = renewal_time;
                            self.dhcp.t2 = rebinding_time;
                            // de-rate lease time by 12%
                            self.dhcp.lease = lease_time.saturating_sub(lease_time / 8);
                            self.dhcp.lease_monotonic_secs = monotonic_secs;

                            info!("dhcp.ip: {}", self.dhcp.ip);

                            self.w5500.set_subr(&subnet_mask).map_err(Error::W5500)?;
                            self.w5500.set_sipr(&self.dhcp.ip).map_err(Error::W5500)?;
                            self.w5500.set_gar(&gateway).map_err(Error::W5500)?;

                            self.dhcp.state = State::Bound;
                            self.dhcp.timeout = None;
                        }
                        MsgType::Nak => {
                            info!("request was NAK'd");
                            self.discover(monotonic_secs)?;
                        }
                        mt => info!("ignoring message type {:?}", mt),
                    }
                }
                state => debug!("ignored IRQ in state={:?}", state),
            }
        }

        Ok(())
    }

    /// Poll the DHCP client.
    ///
    /// This should be called approximately every second.
    pub fn poll(&mut self, monotonic_secs: u32) -> Result<(), Error<E>> {
        match self.dhcp.timeout {
            Some(to) => {
                if monotonic_secs.wrapping_sub(to) > 10 {
                    info!(
                        "timeout waiting for state to transition from {:?}",
                        self.dhcp.state
                    );
                    self.discover(monotonic_secs)?;
                }
            }
            None => match self.dhcp.state {
                State::Init => {
                    self.discover(monotonic_secs)?;
                }
                // states handled by IRQs and timeouts
                State::Selecting | State::Requesting | State::Renewing | State::Rebinding => (),
                // states we do not care about (yet)
                State::InitReboot | State::Rebooting => (),
                State::Bound => {
                    let elapsed: u32 = monotonic_secs.wrapping_sub(self.dhcp.lease_monotonic_secs);
                    if elapsed > self.dhcp.t1 {
                        warn!("t1 expired, taking no action");
                    }
                    if elapsed > self.dhcp.t2 {
                        warn!("t2 expired, taking no action");
                    }
                    if elapsed > self.dhcp.lease {
                        info!("lease expired");
                        self.discover(monotonic_secs)?;
                    }
                }
            },
        }

        Ok(())
    }

    fn discover(&mut self, monotonic_secs: u32) -> Result<(), Error<E>> {
        self.dhcp.ip = Ipv4Addr::UNSPECIFIED;
        self.dhcp.xid = self.dhcp.rand.next_u32();
        debug!("sending DHCPDISCOVER xid={:08X}", self.dhcp.xid);

        let mut pkt = PktSer::new(self.buf);
        let buf: &[u8] = pkt
            .dhcp_discover(&self.dhcp.mac, self.dhcp.hostname, self.dhcp.xid)
            .ok_or(Error::BufOverflow)?;

        self.w5500.set_sipr(&self.dhcp.ip).map_err(Error::W5500)?;
        self.w5500
            .udp_bind(self.dhcp.sn, DHCP_SOURCE_PORT)
            .map_err(Error::W5500)?;

        let tx_bytes: usize = self
            .w5500
            .udp_send_to_if_free(self.dhcp.sn, buf, &DHCP_BROADCAST)
            .map_err(Error::W5500)?;

        if tx_bytes == 0 {
            Err(Error::TxOverflow)
        } else {
            self.dhcp.state = State::Selecting;
            self.dhcp.timeout = Some(monotonic_secs);
            Ok(())
        }
    }

    fn recv(&mut self) -> Result<Option<PktDe>, Error<E>> {
        match self.w5500.udp_peek_from_header(self.dhcp.sn) {
            Ok((len, _)) => {
                if len > self.buf.len() {
                    Err(Error::BufOverflow)
                } else {
                    let (len, src) = nb::block!(self.w5500.udp_recv_from(self.dhcp.sn, self.buf))
                        .map_err(Error::W5500)?;

                    debug!("RX {} B from {}", len, src);

                    let pkt: PktDe = PktDe::new(self.buf);

                    if pkt.is_bootreply() {
                        debug!("packet is not a bootreply");
                        return Ok(None);
                    }

                    let recv_xid: u32 = pkt.xid().ok_or(Error::BufOverflow)?;

                    if recv_xid != self.dhcp.xid {
                        debug!(
                            "recv xid {:08X} does not match ours {:08X}",
                            recv_xid, self.dhcp.xid
                        );
                        return Ok(None);
                    }

                    Ok(Some(pkt))
                }
            }
            Err(WouldBlock) => {
                error!("interrupt is misconfigured");
                Ok(None)
            }
            Err(nb::Error::Other(e)) => Err(Error::W5500(e)),
        }
    }

    fn request(&mut self, monotonic_secs: u32) -> Result<(), Error<E>> {
        self.dhcp.xid = self.dhcp.rand.next_u32();
        debug!("sending DHCPREQUEST xid={:08X}", self.dhcp.xid);

        let mut pkt: PktSer = PktSer::new(self.buf);
        let buf: &[u8] = pkt
            .dhcp_request(
                &self.dhcp.mac,
                &self.dhcp.ip,
                self.dhcp.hostname,
                self.dhcp.xid,
            )
            .ok_or(Error::BufOverflow)?;

        let tx_bytes: usize = self
            .w5500
            .udp_send_if_free(self.dhcp.sn, buf)
            .map_err(Error::W5500)?;

        if tx_bytes == 0 {
            Err(Error::TxOverflow)
        } else {
            self.dhcp.state = State::Requesting;
            self.dhcp.timeout = Some(monotonic_secs);
            Ok(())
        }
    }
}
