//! DHCP client for the [Wiznet W5500] SPI internet offload chip.
//!
//! # Warning
//!
//! Please review the code before use in a production environment.
//! This code has been tested, but only with a single DHCP server.
//!
//! # Feature Flags
//!
//! All features are disabled by default.
//!
//! * `embedded-hal`: Passthrough to [`w5500-hl`].
//! * `std`: Passthrough to [`w5500-hl`].
//! * `defmt`: Enable logging with `defmt`. Also a passthrough to [`w5500-hl`].
//! * `log`: Enable logging with `log`.
//!
//! [`std::net`]: https://doc.rust-lang.org/std/net/index.html
//! [`w5500-hl`]: https://github.com/newAM/w5500-hl-rs
//! [Wiznet W5500]: https://www.wiznet.io/product-item/w5500/
#![cfg_attr(docsrs, feature(doc_cfg), feature(doc_auto_cfg))]
#![cfg_attr(all(not(feature = "std"), not(test)), no_std)]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

mod pkt;
mod rand;

use hl::{
    io::Seek,
    ll::{
        net::{Eui48Addr, Ipv4Addr},
        LinkStatus, PhyCfg, Registers, Sn, SocketInterrupt, SocketInterruptMask,
    },
    net::SocketAddrV4,
    Common, Error, Udp, UdpReader,
};
pub use w5500_hl as hl;
pub use w5500_hl::ll;

use pkt::{send_dhcp_discover, send_dhcp_request, MsgType, PktDe};
pub use w5500_hl::Hostname;

/// DHCP destination port.
pub const DST_PORT: u16 = 67;

/// DHCP source port.
pub const SRC_PORT: u16 = 68;

/// Duration in seconds to wait for physical link-up.
const LINK_UP_TIMEOUT_SECS: u32 = 1;

/// DHCP client states.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive] // may support rebooting and ini-reboot in the future
#[doc(hidden)]
pub enum State {
    /// Initialization state, client sends DHCPDISCOVER.
    Init,
    /// Client waits for DHCPOFFER.
    Selecting,
    /// Client sends for DHCPREQUEST.
    Requesting,
    /// Client has a valid lease.
    Bound,
    /// T1 expires, client sends DHCPREQUEST to renew.
    Renewing,
    /// T2 expires, client sends DHCPREQUEST to rebind.
    Rebinding,
}

/// DHCP client.
///
/// This requires the W5500 interrupt pin configured for a falling edge trigger.
///
/// # Example
///
/// ```no_run
/// use rand_core::RngCore;
/// use w5500_dhcp::{
///     ll::{net::Eui48Addr, Sn},
///     Client, Hostname,
/// };
/// # let mut w5500 = w5500_regsim::W5500::default();
/// # let mut rng = rand_core::OsRng;
/// # fn this_is_where_you_setup_the_w5500_int_pin_for_a_falling_edge_trigger() { }
/// # fn monotonic_seconds() -> u32 { 0 }
///
/// const DHCP_SN: Sn = Sn::Sn0;
///
/// // locally administered MAC address
/// const MAC_ADDRESS: Eui48Addr = Eui48Addr::new(0x02, 0x00, 0x11, 0x22, 0x33, 0x44);
///
/// const HOSTNAME: Hostname = Hostname::new_unwrapped("example");
///
/// this_is_where_you_setup_the_w5500_int_pin_for_a_falling_edge_trigger();
///
/// let seed: u64 = rng.next_u64();
///
/// let mut dhcp: Client = Client::new(DHCP_SN, seed, MAC_ADDRESS, HOSTNAME);
///
/// dhcp.setup_socket(&mut w5500)?;
///
/// let call_after_secs: u32 = dhcp.process(&mut w5500, monotonic_seconds())?;
/// // call process again after call_after_secs, or on the next interrupt
/// # Ok::<(), w5500_hl::Error<std::io::ErrorKind>>(())
/// ```
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Client<'a> {
    /// Socket to use for the DHCP client.
    sn: Sn,
    /// DHCP client state
    state: State,
    /// Instant of the last state transition
    state_timeout: Option<u32>,
    /// Timeout duration in seconds
    timeout: u32,
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
    hostname: Hostname<'a>,
    /// DNS server
    dns: Option<Ipv4Addr>,
    /// (S)NTP server
    ntp: Option<Ipv4Addr>,
    /// Broadcast address
    broadcast_addr: SocketAddrV4,
    /// Source Port
    src_port: u16,
}

impl<'a> Client<'a> {
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
    ///     Client, Hostname,
    /// };
    /// # let mut rng = rand_core::OsRng;
    ///
    /// const DHCP_SN: Sn = Sn::Sn0;
    /// // locally administered MAC address
    /// const MAC_ADDRESS: Eui48Addr = Eui48Addr::new(0x02, 0x00, 0x11, 0x22, 0x33, 0x44);
    /// const HOSTNAME: Hostname = Hostname::new_unwrapped("example");
    /// let seed: u64 = rng.next_u64();
    ///
    /// let dhcp: Client = Client::new(DHCP_SN, seed, MAC_ADDRESS, HOSTNAME);
    /// ```
    pub fn new(sn: Sn, seed: u64, mac: Eui48Addr, hostname: Hostname<'a>) -> Self {
        let mut rand: rand::Rand = rand::Rand::new(seed);

        Self {
            sn,
            state: State::Init,
            state_timeout: None,
            timeout: 5,
            t1: 0,
            t2: 0,
            lease: 0,
            lease_monotonic_secs: 0,
            xid: rand.next_u32(),
            rand,
            mac,
            ip: Ipv4Addr::UNSPECIFIED,
            hostname,
            dns: None,
            ntp: None,
            broadcast_addr: SocketAddrV4::new(Ipv4Addr::BROADCAST, DST_PORT),
            src_port: SRC_PORT,
        }
    }

    /// Set the DHCP state timeout duration in seconds.
    ///
    /// This is the duration to wait for the DHCP server to send a reply before
    /// resetting and starting over.
    ///
    /// # Example
    ///
    /// Set a 10 second timeout.
    ///
    /// ```
    /// use rand_core::RngCore;
    /// use w5500_dhcp::{
    ///     ll::{net::Eui48Addr, Sn},
    ///     Client, Hostname,
    /// };
    /// # let mut rng = rand_core::OsRng;
    ///
    /// const HOSTNAME: Hostname = Hostname::new_unwrapped("example");
    /// let mut dhcp: Client = Client::new(
    ///     Sn::Sn0,
    ///     rng.next_u64(),
    ///     Eui48Addr::new(0x02, 0x00, 0x11, 0x22, 0x33, 0x44),
    ///     HOSTNAME,
    /// );
    /// dhcp.set_timeout_secs(10);
    /// ```
    pub fn set_timeout_secs(&mut self, secs: u32) {
        self.timeout = secs;
    }

    /// Returns `true` if the DHCP client has a valid lease.
    ///
    /// # Example
    ///
    /// ```
    /// use rand_core::RngCore;
    /// use w5500_dhcp::{
    ///     ll::{net::Eui48Addr, Sn},
    ///     Client, Hostname,
    /// };
    /// # let mut rng = rand_core::OsRng;
    ///
    /// const HOSTNAME: Hostname = Hostname::new_unwrapped("example");
    /// let dhcp: Client = Client::new(
    ///     Sn::Sn0,
    ///     rng.next_u64(),
    ///     Eui48Addr::new(0x02, 0x00, 0x11, 0x22, 0x33, 0x44),
    ///     HOSTNAME,
    /// );
    /// assert_eq!(dhcp.has_lease(), false);
    /// ```
    #[inline]
    pub fn has_lease(&self) -> bool {
        matches!(
            self.state,
            State::Bound | State::Rebinding | State::Renewing
        )
    }

    /// Setup the DHCP socket interrupts.
    ///
    /// This should be called once during initialization.
    ///
    /// This only sets up the W5500 interrupts, you must configure the W5500
    /// interrupt pin for a falling edge trigger yourself.
    pub fn setup_socket<W5500: Registers>(&self, w5500: &mut W5500) -> Result<(), W5500::Error> {
        let simr: u8 = w5500.simr()?;
        w5500.set_simr(self.sn.bitmask() | simr)?;
        const MASK: SocketInterruptMask = SocketInterruptMask::ALL_MASKED.unmask_recv();
        w5500.set_sn_imr(self.sn, MASK)?;
        w5500.close(self.sn)?;
        w5500.set_sipr(&self.ip)?;
        w5500.udp_bind(self.sn, self.src_port)
    }

    fn timeout_elapsed_secs(&self, monotonic_secs: u32) -> Option<u32> {
        self.state_timeout.map(|to| monotonic_secs - to)
    }

    fn next_call(&self, monotonic_secs: u32) -> u32 {
        if let Some(timeout_elapsed_secs) = self.timeout_elapsed_secs(monotonic_secs) {
            self.timeout.saturating_sub(timeout_elapsed_secs)
        } else {
            let elapsed: u32 = monotonic_secs.saturating_sub(self.lease_monotonic_secs);
            match self.state {
                State::Bound => self.t1.saturating_sub(elapsed),
                State::Renewing => self.t2.saturating_sub(elapsed),
                // rebinding
                _ => self.lease.saturating_sub(elapsed),
            }
        }
    }

    fn set_state_with_timeout(&mut self, state: State, monotonic_secs: u32) {
        debug!(
            "{:?} -> {:?} with timeout {}",
            self.state, state, monotonic_secs
        );
        self.state = state;
        self.state_timeout = Some(monotonic_secs);
    }

    fn set_state(&mut self, state: State) {
        debug!("{:?} -> {:?} without timeout", self.state, state);
        self.state = state;
        self.state_timeout = None;
    }

    /// Get the DNS server provided by DHCP.
    ///
    /// After the client is bound this will return the IP address of the
    /// most-preferred DNS server.
    /// If the client is not bound, or the DHCP server did not provide this
    /// address it will return `None`.
    #[inline]
    pub fn dns(&self) -> Option<Ipv4Addr> {
        self.dns
    }

    /// Get the NTP server provided by DHCP.
    ///
    /// After the client is bound this will return the IP address of the
    /// most-preferred NTP server.
    /// If the client is not bound, or the DHCP server did not provide this
    /// address it will return `None`.
    #[inline]
    pub fn ntp(&self) -> Option<Ipv4Addr> {
        self.ntp
    }

    /// Process DHCP client events.
    ///
    /// This should be called in these conditions:
    ///
    /// 1. After power-on-reset, to start the DHCP client.
    /// 2. A W5500 interrupt on the DHCP socket is received.
    /// 3. After the duration indicated by the return value.
    ///
    /// This will clear any pending DHCP socket interrupts.
    ///
    /// # System Time
    ///
    /// You must supply a monotonic `u32` that counts the number of seconds
    /// since system boot to this method.
    ///
    /// This is required for timeouts, and tracking the DHCP lease timers.
    ///
    /// # Return Value
    ///
    /// The return value is a number of seconds into the future you should
    /// call this method.
    /// You may call this method before that time, but nothing will happen.
    pub fn process<W5500: Registers>(
        &mut self,
        w5500: &mut W5500,
        monotonic_secs: u32,
    ) -> Result<u32, Error<W5500::Error>> {
        let sn_ir: SocketInterrupt = w5500.sn_ir(self.sn)?;
        if sn_ir.any_raised() {
            w5500.set_sn_ir(self.sn, sn_ir)?;
        }

        if self.state == State::Init {
            let phy_cfg: PhyCfg = w5500.phycfgr()?;
            if phy_cfg.lnk() != LinkStatus::Up {
                debug!("Link is not up: {}", phy_cfg);
                return Ok(LINK_UP_TIMEOUT_SECS);
            };
        }

        fn recv<W5500: Registers>(
            w5500: &mut W5500,
            sn: Sn,
            xid: u32,
        ) -> Result<Option<PktDe<W5500>>, Error<W5500::Error>> {
            let reader: UdpReader<W5500> = match w5500.udp_reader(sn) {
                Ok(r) => r,
                Err(Error::WouldBlock) => return Ok(None),
                Err(e) => return Err(e),
            };

            debug!(
                "RX {} B from {}",
                reader.header().len,
                reader.header().origin
            );

            let stream_len: u16 = reader.stream_len();
            let header_len: u16 = reader.header().len;
            if header_len > stream_len {
                // this is often recoverable
                warn!(
                    "packet may be truncated header={} > stream={}",
                    header_len, stream_len
                );
                return Ok(None);
            }

            let mut pkt: PktDe<W5500> = PktDe::from(reader);
            if pkt.is_bootreply()? {
                debug!("packet is not a bootreply");
                return Ok(None);
            }
            let recv_xid: u32 = pkt.xid()?;
            if recv_xid != xid {
                debug!("recv xid {:08X} does not match ours {:08X}", recv_xid, xid);
                return Ok(None);
            }

            Ok(Some(pkt))
        }

        while let Some(mut pkt) = recv(w5500, self.sn, self.xid)? {
            debug!("{:?}", self.state);
            match self.state {
                State::Selecting => {
                    self.ip = pkt.yiaddr()?;
                    pkt.done()?;
                    self.request(w5500)?;
                    self.set_state_with_timeout(State::Requesting, monotonic_secs);
                }
                State::Requesting | State::Renewing | State::Rebinding => {
                    match pkt.msg_type()? {
                        Some(MsgType::Ack) => {
                            let subnet_mask: Option<Ipv4Addr> = pkt.subnet_mask()?;
                            let gateway: Option<Ipv4Addr> = pkt.dhcp_server()?;
                            let dns: Option<Ipv4Addr> = pkt.dns()?;
                            let ntp: Option<Ipv4Addr> = pkt.ntp()?;

                            let renewal_time: u32 = match pkt.renewal_time()? {
                                Some(x) => x,
                                None => {
                                    error!("renewal_time option missing");
                                    return Ok(self.next_call(monotonic_secs));
                                }
                            };
                            info!("renewal_time: {}", renewal_time);
                            let rebinding_time: u32 = match pkt.rebinding_time()? {
                                Some(x) => x,
                                None => {
                                    error!("rebinding_time option missing");
                                    return Ok(self.next_call(monotonic_secs));
                                }
                            };
                            info!("rebinding_time: {}", rebinding_time);
                            let lease_time: u32 = match pkt.lease_time()? {
                                Some(x) => x,
                                None => {
                                    error!("lease_time option missing");
                                    return Ok(self.next_call(monotonic_secs));
                                }
                            };
                            info!("lease_time: {}", lease_time);

                            // de-rate times by 12%
                            self.t1 = renewal_time.saturating_sub(renewal_time / 8);
                            self.t2 = rebinding_time.saturating_sub(rebinding_time / 8);
                            self.lease = lease_time.saturating_sub(lease_time / 8);
                            self.lease_monotonic_secs = monotonic_secs;

                            pkt.done()?;

                            match subnet_mask {
                                Some(subnet_mask) => {
                                    info!("subnet_mask: {}", subnet_mask);
                                    w5500.set_subr(&subnet_mask)?;
                                }
                                None if self.state == State::Renewing => (),
                                None => {
                                    error!("subnet_mask option missing");
                                    return Ok(self.next_call(monotonic_secs));
                                }
                            };

                            match gateway {
                                Some(gateway) => {
                                    info!("gateway: {}", gateway);
                                    w5500.set_gar(&gateway)?;
                                }
                                None if self.state == State::Renewing => (),
                                None => {
                                    error!("gateway option missing");
                                    return Ok(self.next_call(monotonic_secs));
                                }
                            };

                            // rebinding and renewal do not need to set a new IP
                            if self.state == State::Requesting {
                                info!("dhcp.ip: {}", self.ip);
                                w5500.set_sipr(&self.ip)?;
                            }

                            if let Some(dns) = dns {
                                info!("DNS: {}", dns);
                                self.dns.replace(dns);
                            };

                            if let Some(ntp) = ntp {
                                info!("NTP: {}", ntp);
                                self.ntp.replace(ntp);
                            };

                            self.set_state(State::Bound);
                        }
                        Some(MsgType::Nak) => {
                            info!("request was NAK'd");
                            pkt.done()?;
                            self.discover(w5500, monotonic_secs)?;
                        }
                        Some(mt) => {
                            info!("ignoring message type {:?}", mt);
                            pkt.done()?;
                        }
                        None => {
                            error!("message type option missing");
                            pkt.done()?;
                        }
                    }
                }
                state => {
                    debug!("ignored IRQ in state={:?}", state);
                    pkt.done()?;
                }
            }
        }

        if let Some(elapsed_secs) = self.timeout_elapsed_secs(monotonic_secs) {
            if elapsed_secs > self.timeout {
                info!(
                    "timeout waiting for state to transition from {:?}",
                    self.state
                );
                self.discover(w5500, monotonic_secs)?;
            }
        } else {
            match self.state {
                State::Init => self.discover(w5500, monotonic_secs)?,
                // states handled by IRQs and timeouts
                State::Selecting | State::Requesting => (),
                State::Bound | State::Renewing | State::Rebinding => {
                    let elapsed: u32 = monotonic_secs.wrapping_sub(self.lease_monotonic_secs);
                    if elapsed > self.lease {
                        info!("lease expired");
                        self.discover(w5500, monotonic_secs)?;
                    } else if elapsed > self.t2
                        && matches!(self.state, State::Bound | State::Renewing)
                    {
                        info!("t2 expired");
                        self.request(w5500)?;
                        // no need for timeout, lease expiration will handle failures
                        self.set_state(State::Rebinding);
                    } else if elapsed > self.t1 && matches!(self.state, State::Bound) {
                        info!("t1 expired");
                        self.request(w5500)?;
                        // no need for timeout, t2 expiration will handle failures
                        self.set_state(State::Renewing);
                    }
                }
            }
        }

        Ok(self.next_call(monotonic_secs))
    }

    fn discover<W5500: Registers>(
        &mut self,
        w5500: &mut W5500,
        monotonic_secs: u32,
    ) -> Result<(), Error<W5500::Error>> {
        self.ip = Ipv4Addr::UNSPECIFIED;
        self.xid = self.rand.next_u32();
        debug!("sending DHCPDISCOVER xid={:08X}", self.xid);

        w5500.set_sipr(&self.ip)?;
        w5500.udp_bind(self.sn, self.src_port)?;

        send_dhcp_discover(
            w5500,
            self.sn,
            &self.mac,
            self.hostname,
            self.xid,
            &self.broadcast_addr,
        )?;
        self.set_state_with_timeout(State::Selecting, monotonic_secs);
        Ok(())
    }

    fn request<W5500: Registers>(&mut self, w5500: &mut W5500) -> Result<(), Error<W5500::Error>> {
        self.xid = self.rand.next_u32();
        debug!("sending DHCPREQUEST xid={:08X}", self.xid);
        send_dhcp_request(w5500, self.sn, &self.mac, &self.ip, self.hostname, self.xid)?;
        Ok(())
    }

    /// Set the DHCP source port.
    ///
    /// Defaults to [`SRC_PORT`].
    /// This is an interface for testing, typically the default is what you
    /// want to use.
    #[inline]
    #[doc(hidden)]
    pub fn set_src_port(&mut self, port: u16) {
        self.src_port = port
    }

    /// Set the client broadcast address.
    ///
    /// Defaults to [`Ipv4Addr::BROADCAST`]:[`DST_PORT`].
    /// This is an interface for testing, typically the default is what you
    /// want to use.
    #[inline]
    #[doc(hidden)]
    pub fn set_broadcast_addr(&mut self, addr: SocketAddrV4) {
        self.broadcast_addr = addr;
    }

    /// DHCP client state.
    #[inline]
    #[doc(hidden)]
    pub fn state(&self) -> State {
        self.state
    }

    /// T1 time.
    ///
    /// This should be used only as a debug interface, and not to set timers.
    /// DHCP timers are tracked internally by [`process`](Self::process).
    ///
    /// Returns `None` if the DHCP client does not have a valid lease.
    #[inline]
    #[doc(hidden)]
    pub fn t1(&self) -> Option<u32> {
        match self.state {
            State::Init | State::Selecting | State::Requesting => None,
            State::Bound | State::Renewing | State::Rebinding => Some(self.t1),
        }
    }

    /// T2 time.
    ///
    /// This should be used only as a debug interface, and not to set timers.
    /// DHCP timers are tracked internally by [`process`](Self::process).
    ///
    /// Returns `None` if the DHCP client does not have a valid lease.
    #[inline]
    #[doc(hidden)]
    pub fn t2(&self) -> Option<u32> {
        match self.state {
            State::Init | State::Selecting | State::Requesting => None,
            State::Bound | State::Renewing | State::Rebinding => Some(self.t2),
        }
    }

    /// Lease time.
    ///
    /// This should be used only as a debug interface, and not to set timers.
    /// DHCP timers are tracked internally by [`process`](Self::process).
    ///
    /// Returns `None` if the DHCP client does not have a valid lease.
    #[inline]
    #[doc(hidden)]
    pub fn lease_time(&self) -> Option<u32> {
        match self.state {
            State::Init | State::Selecting | State::Requesting => None,
            State::Bound | State::Renewing | State::Rebinding => Some(self.lease),
        }
    }
}
