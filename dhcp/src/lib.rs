//! Simple DHCP client for the [Wiznet W5500] SPI internet offload chip.
//!
//! # Warning
//!
//! Please review the code before use in a production environment.
//!
//! The code has only been tested with a single DHCP server, and has not gone
//! through any fuzzing.
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
//! * `embedded-hal`: Passthrough to [`w5500-hl`].
//! * `std`: Passthrough to [`w5500-hl`].
//! * `defmt`: Enable logging with `defmt`.  Mutually exclusive with `log`.
//!   Also a passthrough to [`w5500-hl`].
//! * `log`: Enable logging with `log`.  Mutually exclusive with `defmt`.
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
    ll::{
        net::{Eui48Addr, Ipv4Addr, SocketAddrV4},
        LinkStatus, PhyCfg, Registers, Sn, SocketInterrupt, SocketInterruptMask,
    },
    Common, Error, Udp, UdpReader,
};
pub use w5500_hl as hl;
pub use w5500_hl::ll;

use pkt::{send_dhcp_discover, send_dhcp_request, MsgType, PktDe};
pub use w5500_hl::Hostname;

/// DHCP destination port.
#[cfg(target_os = "none")]
pub const DHCP_DESTINATION_PORT: u16 = 67;
/// DHCP destination port for testing on `std` targets.
#[cfg(not(target_os = "none"))]
pub const DHCP_DESTINATION_PORT: u16 = 2050;

/// DHCP source port.
#[cfg(target_os = "none")]
pub const DHCP_SOURCE_PORT: u16 = 68;
/// DHCP source port for testing on `std` targets.
#[cfg(not(target_os = "none"))]
pub const DHCP_SOURCE_PORT: u16 = 2051;

#[cfg(target_os = "none")]
const DHCP_BROADCAST: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::BROADCAST, DHCP_DESTINATION_PORT);
#[cfg(not(target_os = "none"))]
const DHCP_BROADCAST: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::LOCALHOST, DHCP_DESTINATION_PORT);

/// Duration in seconds to wait for the DHCP server to send a response.
const TIMEOUT_SECS: u32 = 10;

/// Duration in seconds to wait for physical link-up.
const LINK_UP_TIMEOUT_SECS: u32 = 2;

/// DHCP client states.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[allow(dead_code)]
enum State {
    Init,
    Selecting,
    Requesting,
    InitReboot,
    Rebooting,
    Bound,
    Renewing,
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
/// // unwrap is not `const` yet
/// const HOSTNAME: Hostname = match Hostname::new("example") {
///     Some(hn) => hn,
///     None => panic!("invalid hostname"),
/// };
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
/// # Ok::<(), w5500_hl::Error<std::io::Error>>(())
/// ```
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Client<'a> {
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
    hostname: Hostname<'a>,
    /// DNS server
    dns: Option<Ipv4Addr>,
    /// (S)NTP server
    ntp: Option<Ipv4Addr>,
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
    /// // unwrap is not `const` yet
    /// const HOSTNAME: Hostname = match Hostname::new("example") {
    ///     Some(hn) => hn,
    ///     None => panic!("invalid hostname"),
    /// };
    /// let seed: u64 = rng.next_u64();
    ///
    /// let dhcp: Client = Client::new(DHCP_SN, seed, MAC_ADDRESS, HOSTNAME);
    /// ```
    pub fn new(sn: Sn, seed: u64, mac: Eui48Addr, hostname: Hostname<'a>) -> Self {
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
            dns: None,
            ntp: None,
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
    ///     Client, Hostname,
    /// };
    /// # let mut rng = rand_core::OsRng;
    ///
    /// let dhcp: Client = Client::new(
    ///     Sn::Sn0,
    ///     rng.next_u64(),
    ///     Eui48Addr::new(0x02, 0x00, 0x11, 0x22, 0x33, 0x44),
    ///     Hostname::new("demo").unwrap(),
    /// );
    /// assert_eq!(dhcp.is_bound(), false);
    /// ```
    #[inline]
    pub fn is_bound(&self) -> bool {
        self.state == State::Bound
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
        w5500.udp_bind(self.sn, DHCP_SOURCE_PORT)
    }

    fn timeout_elapsed_secs(&self, monotonic_secs: u32) -> Option<u32> {
        self.timeout.map(|to| monotonic_secs - to)
    }

    fn next_call(&self, monotonic_secs: u32) -> u32 {
        if let Some(timeout_elapsed_secs) = self.timeout_elapsed_secs(monotonic_secs) {
            TIMEOUT_SECS.saturating_sub(timeout_elapsed_secs)
        } else {
            self.lease
                .saturating_sub(monotonic_secs.saturating_sub(self.lease_monotonic_secs))
        }
        .saturating_add(1)
    }

    /// Get the DNS server provided by DHCP.
    ///
    /// After the client is bound this will return the IP address of the
    /// most-preferred DNS server.
    ///
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
    ///
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
            match self.state {
                State::Selecting => {
                    self.ip = pkt.yiaddr()?;
                    pkt.done()?;
                    self.request(w5500, monotonic_secs)?;
                }
                State::Requesting | State::Renewing | State::Rebinding => {
                    match pkt.msg_type()? {
                        Some(MsgType::Ack) => {
                            let subnet_mask: Ipv4Addr = match pkt.subnet_mask()? {
                                Some(x) => x,
                                None => {
                                    error!("subnet_mask option missing");
                                    return Ok(self.next_call(monotonic_secs));
                                }
                            };
                            info!("subnet_mask: {}", subnet_mask);
                            let gateway: Ipv4Addr = match pkt.dhcp_server()? {
                                Some(x) => x,
                                None => {
                                    error!("gateway option missing");
                                    return Ok(self.next_call(monotonic_secs));
                                }
                            };
                            info!("gateway: {}", gateway);
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

                            if let Some(dns) = pkt.dns()? {
                                info!("DNS: {}", dns);
                                self.dns.replace(dns);
                            };
                            if let Some(ntp) = pkt.ntp()? {
                                info!("NTP: {}", ntp);
                                self.ntp.replace(ntp);
                            };

                            self.t1 = renewal_time;
                            self.t2 = rebinding_time;
                            // de-rate lease time by 12%
                            self.lease = lease_time.saturating_sub(lease_time / 8);
                            self.lease_monotonic_secs = monotonic_secs;

                            info!("dhcp.ip: {}", self.ip);

                            pkt.done()?;
                            w5500.set_subr(&subnet_mask)?;
                            w5500.set_sipr(&self.ip)?;
                            w5500.set_gar(&gateway)?;

                            self.state = State::Bound;
                            self.timeout = None;
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
            if elapsed_secs > TIMEOUT_SECS {
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
                State::Selecting | State::Requesting | State::Renewing | State::Rebinding => (),
                // states we do not care about (yet)
                State::InitReboot | State::Rebooting => (),
                State::Bound => {
                    let elapsed: u32 = monotonic_secs.wrapping_sub(self.lease_monotonic_secs);
                    if elapsed > self.t1 {
                        warn!("t1 expired, taking no action");
                    }
                    if elapsed > self.t2 {
                        warn!("t2 expired, taking no action");
                    }
                    if elapsed > self.lease {
                        info!("lease expired");
                        self.discover(w5500, monotonic_secs)?;
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
        w5500.udp_bind(self.sn, DHCP_SOURCE_PORT)?;

        send_dhcp_discover(w5500, self.sn, &self.mac, self.hostname, self.xid)?;
        self.state = State::Selecting;
        self.timeout = Some(monotonic_secs);
        Ok(())
    }

    fn request<W5500: Registers>(
        &mut self,
        w5500: &mut W5500,
        monotonic_secs: u32,
    ) -> Result<(), Error<W5500::Error>> {
        self.xid = self.rand.next_u32();
        debug!("sending DHCPREQUEST xid={:08X}", self.xid);

        send_dhcp_request(w5500, self.sn, &self.mac, &self.ip, self.hostname, self.xid)?;

        self.state = State::Requesting;
        self.timeout = Some(monotonic_secs);
        Ok(())
    }
}
