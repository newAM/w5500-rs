//! Register simulation for the [Wiznet W5500] internet offload chip.
//!
//! This implements the [`w5500_ll::Registers`] trait using [`std::net`] sockets
//! to simulate the W5500 on your local PC.
//!
//! This is a best-effort implementation to aid in development of application
//! code, not all features of the W5500 will be fully simulated.
//!
//! # Notes
//!
//! This is in an early alpha state, there are many todos throughout the code.
//!
//! ## Not-implemented
//!
//! * MR (Mode Register)
//!     * Wake on LAN
//!     * Ping block
//!     * PPPoE mode
//!     * Force ARP
//! * INTLEVEL (Interrupt Low Level Timer Register)
//! * IR (Interrupt Register)
//! * IMR (Interrupt Mask Register)
//! * GAR (Gateway IP Address Register)
//! * SUBR (Subnet Mask Register)
//! * SHAR (Source Hardware Address Register)
//! * SIPR (Source IP Address Register)
//! * INTLEVEL (Interrupt Low Level Timer Register)
//! * IR (Interrupt Register)
//! * IMR (Interrupt Mask Register)
//! * SIR (Socket Interrupt Register)
//!     * Partial; see SN_IR
//! * SIMR (Socket Interrupt Mask Register)
//! * RTR (Retry Time Register)
//! * RCR (Retry Count Register)
//! * PTIMER (PPP LCP Request Timer Register)
//! * PMAGIC (PPP LCP Magic Number Register)
//! * PHAR (PPP Destination MAC Address Register)
//! * PSID (PPP Session Identification Register)
//! * PMRU (PPP Maximum Segment Size Register)
//! * UIPR (Unreachable IP Address Register)
//! * UPORT (Unreachable Port Register)
//! * PHYCFGR (PHY Configuration Register)
//! * SN_MR (Socket n Mode Register)
//! * SN_IR (Socket n Interrupt Register)
//!     * DISCON
//!     * TIMEOUT
//!     * SENDOK
//! * SN_SR (Socket n Status Register)
//!     * SynSent
//!     * SynRecv
//!     * FinWait
//!     * Closing
//!     * TimeWait
//!     * CloseWait
//!     * LastAck
//!     * Macraw
//! * SN_MSSR (Socket n Maximum Segment Size Register)
//! * SN_TOS (Socket n IP TOS Register)
//! * SN_TTL (Socket n IP TTL)
//! * SN_RXBUF_SIZE (Socket n Receive Buffer Size Register)
//! * SN_TXBUF_SIZE (Socket n Transmit Buffer Size Register)
//! * SN_IMR (Socket n Interrupt Mask Register)
//! * SN_FRAG (Socket n Fragment Offset in IP Header Register)
//! * SN_KPALVTR (Socket n Keep Alive Timer Register)
//!
//! Believe it or not that is not simply a list of all registers.
//!
//! [Wiznet W5500]: https://www.wiznet.io/product-item/w5500/
//! [`std::net`]: https://doc.rust-lang.org/std/net/index.html
//! [`w5500-hl`]: https://crates.io/crates/w5500-hl
//! [`w5500_ll::Registers`]: https://docs.rs/w5500-ll/latest/w5500_ll/trait.Registers.html

use std::{
    io::{self, Read, Write},
    net::{SocketAddrV4, TcpListener, TcpStream, UdpSocket},
};

pub use w5500_ll::{self, Registers};

use w5500_ll::{
    net::{Eui48Addr, Ipv4Addr},
    BufferSize, Mode, Protocol, Reg, Sn, SnReg, SocketCommand, SocketInterrupt, SocketMode,
    SocketStatus, SOCKETS, VERSION,
};

// Socket spacing between blocks.
const SOCKET_SPACING: u8 = 0x04;

const NUM_SOCKETS: usize = SOCKETS.len();
const DEFAULT_BUF_SIZE: usize = BufferSize::KB2.size_in_bytes();

enum SocketType {
    Udp(UdpSocket),
    TcpListener(TcpListener),
    TcpStream(TcpStream),
}

#[derive(PartialEq, Eq)]
enum BlockType {
    Common,
    Socket(Sn),
    Tx(Sn),
    Rx(Sn),
}
impl std::fmt::Display for BlockType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlockType::Common => write!(f, "REG"),
            BlockType::Socket(n) => {
                write!(f, "SN{}", u8::from(*n))
            }
            BlockType::Tx(n) => {
                write!(f, "TX{}", u8::from(*n))
            }
            BlockType::Rx(n) => {
                write!(f, "RX{}", u8::from(*n))
            }
        }
    }
}

fn block_type(block: u8) -> BlockType {
    if block == 0 {
        BlockType::Common
    } else {
        let sn_val: u8 = block / SOCKET_SPACING;
        let sn: Sn = Sn::try_from(sn_val)
            .unwrap_or_else(|_| panic!("Invalid block address: 0x{:02X}", block));
        match block - (sn_val * SOCKET_SPACING) {
            1 => BlockType::Socket(sn),
            2 => BlockType::Tx(sn),
            3 => BlockType::Rx(sn),
            _ => panic!("Invalid block address: 0x{:02X}", block),
        }
    }
}

struct CommonRegs {
    mr: u8,
    gar: Ipv4Addr,
    subr: Ipv4Addr,
    shar: Eui48Addr,
    sipr: Ipv4Addr,
    intlevel: u16,
    ir: u8,
    imr: u8,
    sir: u8,
    simr: u8,
    rtr: u16,
    rcr: u8,
    ptimer: u8,
    pmagic: u8,
    phar: Eui48Addr,
    psid: u16,
    pmru: u16,
    uipr: Ipv4Addr,
    uportr: u16,
    phycfgr: u8,
    versionr: u8,
}

impl CommonRegs {
    /// Reset value of the common registers.
    const RESET: Self = Self {
        mr: 0x00,
        gar: Ipv4Addr::UNSPECIFIED,
        subr: Ipv4Addr::UNSPECIFIED,
        shar: Eui48Addr::UNSPECIFIED,
        sipr: Ipv4Addr::UNSPECIFIED,
        intlevel: 0x00,
        ir: 0x00,
        imr: 0x00,
        sir: 0x00,
        simr: 0x00,
        rtr: 0x07D0,
        rcr: 0x08,
        ptimer: 0x0028,
        pmagic: 0x00,
        phar: Eui48Addr::UNSPECIFIED,
        psid: 0x00,
        pmru: 0xFFFF,
        uipr: Ipv4Addr::UNSPECIFIED,
        uportr: 0x0000,
        phycfgr: 0b10111111,
        versionr: VERSION,
    };
}

struct SocketRegs {
    mr: u8,
    cr: u8,
    ir: SocketInterrupt,
    sr: SocketStatus,
    port: u16,
    dhar: Eui48Addr,
    dipr: Ipv4Addr,
    dport: u16,
    mssr: u16,
    tos: u8,
    ttl: u8,
    rxbuf_size: BufferSize,
    txbuf_size: BufferSize,
    tx_fsr: u16,
    tx_rd: u16,
    tx_wr: u16,
    rx_rsr: u16,
    rx_rd: u16,
    rx_wr: u16,
    imr: u8,
    frag: u16,
    kpalvtr: u8,
}

impl SocketRegs {
    /// Reset value of the socket registers.
    const RESET: Self = Self {
        mr: 0x00,
        cr: 0x00,
        ir: SocketInterrupt::DEFAULT,
        sr: SocketStatus::Closed,
        port: 0x0000,
        dhar: Eui48Addr::new(0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF),
        dipr: Ipv4Addr::UNSPECIFIED,
        dport: 0x00,
        mssr: 0x0000,
        tos: 0x00,
        ttl: 0x80,
        rxbuf_size: BufferSize::KB2,
        txbuf_size: BufferSize::KB2,
        tx_fsr: 0x0800,
        tx_rd: 0x0000,
        tx_wr: 0x0000,
        rx_rsr: 0x0000,
        rx_rd: 0x0000,
        rx_wr: 0x0000,
        imr: 0xFF,
        frag: 0x4000,
        kpalvtr: 0x00,
    };

    pub fn dest(&self) -> std::net::SocketAddrV4 {
        SocketAddrV4::new(self.dipr.into(), self.dport)
    }
}

struct Socket {
    regs: SocketRegs,
    tx_buf: Vec<u8>,
    rx_buf: Vec<u8>,
    inner: Option<SocketType>,
    client: Option<TcpStream>,
}

impl Default for Socket {
    fn default() -> Self {
        Self {
            regs: SocketRegs::RESET,
            tx_buf: vec![0; DEFAULT_BUF_SIZE],
            rx_buf: vec![0; DEFAULT_BUF_SIZE],
            inner: None,
            client: None,
        }
    }
}

/// Simulated W5500.
pub struct W5500 {
    regs: CommonRegs,
    sn: [Socket; NUM_SOCKETS],
}

impl W5500 {
    fn reset(&mut self) {
        *self = Self::default();
    }

    fn socket_cmd_open(&mut self, sn: Sn) -> io::Result<()> {
        let socket = self.socket_mut(sn);

        // These registers are initialized by the OPEN command
        socket.regs.rx_wr = 0;
        socket.regs.rx_rd = 0;
        socket.regs.tx_rd = 0;
        socket.regs.tx_wr = 0;

        let mr = SocketMode::from(socket.regs.mr);

        match mr.protocol() {
            Ok(Protocol::Closed) => log::error!(
                "[{:?}] ignoring OPEN command, socket protocol is not yet",
                sn
            ),
            Ok(Protocol::Tcp) => {
                socket.inner = None;
                self.sim_set_sn_sr(sn, SocketStatus::Init);
            }
            Ok(Protocol::Udp) => {
                let local =
                    SocketAddrV4::new(std::net::Ipv4Addr::new(0, 0, 0, 0), socket.regs.dport);
                log::info!("[{:?}] binding UDP socket to {}", sn, local);

                match UdpSocket::bind(local) {
                    Ok(udp_socket) => {
                        log::info!("[{:?}] bound to {}", sn, local);
                        udp_socket.set_nonblocking(true)?;
                        socket.inner = Some(SocketType::Udp(udp_socket));
                        self.sim_set_sn_sr(sn, SocketStatus::Udp);
                    }
                    Err(e) => {
                        log::warn!("[{:?}] failed to bind socket {}: {}", sn, local, e);
                        self.sim_set_sn_sr(sn, SocketStatus::Closed);
                    }
                }
            }
            Ok(Protocol::Macraw) => {
                if sn == Sn::Sn0 {
                    unimplemented!("MACRAW")
                } else {
                    log::error!(
                        "[{:?}] ignoring OPEN command, MACRAW can only be used on Sn0",
                        sn
                    )
                }
            }
            Err(x) => log::error!(
                "[{:?}] ignoring OPEN command, invalid protocol bits {:#02X}",
                sn,
                x
            ),
        }
        Ok(())
    }

    fn socket_cmd_connect(&mut self, sn: Sn) -> io::Result<()> {
        let socket = self.socket_mut(sn);
        assert_eq!(socket.regs.sr, SocketStatus::Init);

        let addr = socket.regs.dest();
        log::info!("[{:?}] opening a TCP stream to {}", sn, addr);

        match TcpStream::connect(addr) {
            Ok(stream) => {
                log::info!("[{:?}] established TCP connection with {}", sn, addr);
                stream.set_nonblocking(true)?;
                socket.inner = Some(SocketType::TcpStream(stream));
                self.raise_sn_ir(sn, SocketInterrupt::CON_MASK);
                self.sim_set_sn_sr(sn, SocketStatus::Established);
            }
            Err(e) => {
                log::warn!("[{:?}] TCP stream to {} failed: {}", sn, addr, e);
                self.raise_sn_ir(sn, SocketInterrupt::DISCON_MASK);
                self.sim_set_sn_sr(sn, SocketStatus::Closed);
            }
        }

        Ok(())
    }

    fn socket_cmd_listen(&mut self, sn: Sn) -> io::Result<()> {
        let socket = self.socket_mut(sn);
        assert_eq!(socket.regs.sr, SocketStatus::Init);

        let addr = std::net::SocketAddrV4::new(std::net::Ipv4Addr::LOCALHOST, socket.regs.port);
        log::info!("[{:?}] Opening a TCP listener on port {}", sn, addr);
        match TcpListener::bind(addr) {
            Ok(listener) => {
                log::info!("[{:?}] Bound listener on {}", sn, addr);
                listener.set_nonblocking(true)?;
                socket.inner = Some(SocketType::TcpListener(listener));
                self.sim_set_sn_sr(sn, SocketStatus::Listen);
            }
            Err(e) => {
                log::warn!("[{:?}] TCP listener failed to bind to {}: {}", sn, addr, e);
                self.sim_set_sn_sr(sn, SocketStatus::Closed);
                self.raise_sn_ir(sn, SocketInterrupt::TIMEOUT_MASK);
            }
        }

        Ok(())
    }

    fn socket(&self, sn: Sn) -> &Socket {
        &self.sn[usize::from(sn)]
    }

    fn socket_mut(&mut self, sn: Sn) -> &mut Socket {
        &mut self.sn[usize::from(sn)]
    }

    fn sim_set_sn_sr(&mut self, sn: Sn, status: SocketStatus) {
        let socket: &mut Socket = self.socket_mut(sn);
        let old: SocketStatus = socket.regs.sr;
        socket.regs.sr = status;
        if old != status {
            log::info!("[{:?}] {:?} -> {:?}", sn, old, status);
        }
    }

    fn socket_cmd_close(&mut self, sn: Sn) {
        let socket = self.socket_mut(sn);
        socket.inner = None;
        self.sim_set_sn_sr(sn, SocketStatus::Closed);
    }

    fn socket_cmd_send(&mut self, sn: Sn) -> io::Result<()> {
        let socket = self.socket_mut(sn);
        let tail: usize = socket.regs.tx_rd.into();
        let head: usize = socket.regs.tx_wr.into();
        if head == tail {
            log::error!("[{:?}] nothing to send", sn);
            return Ok(());
        }
        let size: usize = if head >= tail {
            head - tail
        } else {
            usize::from(u16::MAX) + head - tail
        };

        log::debug!("[{:?}] tx_rd=0x{:04X}", sn, tail);
        log::debug!("[{:?}] tx_wr=0x{:04X}", sn, head);
        log::debug!("[{:?}] size=0x{:04X}", sn, size);

        debug_assert!(
            size <= socket.regs.txbuf_size.size_in_bytes(),
            "[{:?}] Send data size exceeds buffer size",
            sn
        );

        let mut local_tx_buf: Vec<u8> = Vec::with_capacity(size);

        // convert the circular buffer to somthing more usable
        if head >= tail {
            for buffer_adr in tail..head {
                let buf_idx = buffer_adr % socket.tx_buf.len();
                local_tx_buf.push(socket.tx_buf[buf_idx]);
            }
        } else {
            for buffer_adr in tail..usize::from(u16::MAX) {
                let buf_idx = buffer_adr % socket.tx_buf.len();
                local_tx_buf.push(socket.tx_buf[buf_idx]);
            }
            for buffer_adr in 0..head {
                let buf_idx = buffer_adr % socket.tx_buf.len();
                local_tx_buf.push(socket.tx_buf[buf_idx]);
            }
        }

        debug_assert!(!local_tx_buf.is_empty());

        let dest = socket.regs.dest();

        match socket.inner {
            Some(SocketType::TcpStream(ref mut stream)) => {
                stream.write_all(&local_tx_buf)?;
                stream.flush()?;
            }
            Some(SocketType::Udp(ref mut udp)) => {
                log::info!("[{:?}] sending to {}", sn, dest);
                let num: usize = udp.send_to(&local_tx_buf, &dest)?;
                assert_eq!(num, local_tx_buf.len());
            }
            Some(SocketType::TcpListener(_)) => {
                if let Some(ref mut stream) = socket.client {
                    stream.write_all(&local_tx_buf)?;
                    stream.flush()?;
                }
            }
            None => {
                panic!("[{:?}] Unable to send data, socket is closed", sn)
            }
        }

        Ok(())
    }

    /// The RECV command is used to indicate that the microcontroller has read
    /// an ammount of data from the W5500, as indicated by the `sn_rx_rd`
    /// pointer.
    fn socket_cmd_recv(&mut self, sn: Sn) -> io::Result<()> {
        let socket = self.socket_mut(sn);
        socket.regs.rx_rsr = {
            if socket.regs.rx_wr >= socket.regs.rx_rd {
                socket.regs.rx_wr - socket.regs.rx_rd
            } else {
                u16::try_from(socket.regs.rxbuf_size.size_in_bytes()).unwrap() - socket.regs.rx_wr
                    + socket.regs.rx_rd
            }
        };

        Ok(())
    }

    fn sim_set_sn_rx_buf(&mut self, sn: Sn, data: &[u8]) {
        let socket = self.socket_mut(sn);
        let buf_len: usize = socket.rx_buf.len();

        for byte in data.iter() {
            let buf_idx: usize = usize::from(socket.regs.rx_wr) % buf_len;
            socket.rx_buf[buf_idx] = *byte;
            socket.regs.rx_wr = socket.regs.rx_wr.wrapping_add(1);
            match socket.regs.rx_rsr.checked_add(1) {
                Some(rsr) => socket.regs.rx_rsr = rsr,
                None => {
                    log::warn!("[{:?}] RX buffer overflow", sn);
                    return;
                }
            }
        }
    }

    fn raise_sn_ir(&mut self, sn: Sn, int: u8) {
        self.regs.sir |= sn.bitmask();
        self.socket_mut(sn).regs.ir =
            SocketInterrupt::from(u8::from(self.socket(sn).regs.ir) | int);
    }

    fn check_socket(&mut self, sn: Sn) -> io::Result<()> {
        let socket = self.socket_mut(sn);
        let bufsize: usize = socket.regs.rxbuf_size.size_in_bytes();
        let mut buf: Vec<u8> = vec![0; bufsize];

        match socket.inner {
            Some(SocketType::Udp(ref udp)) => match udp.recv_from(&mut buf) {
                Ok((num, origin)) => {
                    let origin = match origin {
                        std::net::SocketAddr::V4(origin) => origin,
                        other => {
                            panic!(
                                "Internal error, got a non-IPV4 addr from recv_from: {:?}",
                                other
                            )
                        }
                    };
                    log::info!("[{:?}] recv datagram of len {} from {}", sn, num, origin);
                    let numu16 = u16::try_from(num).unwrap();
                    // write out the header
                    self.sim_set_sn_rx_buf(sn, &origin.ip().octets());
                    self.sim_set_sn_rx_buf(sn, &origin.port().to_be_bytes());
                    self.sim_set_sn_rx_buf(sn, &numu16.to_be_bytes());
                    // write the rest of the data
                    self.sim_set_sn_rx_buf(sn, &buf[..num]);
                    self.raise_sn_ir(sn, SocketInterrupt::RECV_MASK);
                    log::warn!("TODO: shorten buf by 8 for a UDP socket");
                }
                Err(e) => match e.kind() {
                    io::ErrorKind::WouldBlock => {}
                    _ => return Err(e),
                },
            },
            Some(SocketType::TcpStream(ref mut stream)) => match stream.read(&mut buf) {
                Ok(num @ 1..=usize::MAX) => {
                    log::info!("[{:?}] recv {} bytes", sn, num);
                    self.sim_set_sn_rx_buf(sn, &buf[..num]);
                    self.raise_sn_ir(sn, SocketInterrupt::RECV_MASK);
                }
                Err(e) => match e.kind() {
                    io::ErrorKind::WouldBlock => {}
                    _ => return Err(e),
                },
                _ => {}
            },
            Some(SocketType::TcpListener(ref mut listener)) => {
                if let Some(ref mut stream) = socket.client {
                    match stream.read(&mut buf) {
                        Ok(num @ 1..=usize::MAX) => {
                            log::info!("[{:?}] recv {} bytes", sn, num);
                            self.sim_set_sn_rx_buf(sn, &buf[..num]);
                            self.raise_sn_ir(sn, SocketInterrupt::RECV_MASK);
                        }
                        Err(e) => match e.kind() {
                            io::ErrorKind::WouldBlock => {}
                            _ => return Err(e),
                        },
                        _ => {}
                    }
                } else {
                    match listener.accept() {
                        Ok((stream, addr)) => {
                            log::info!("[{:?}] Accepted a new stream from {}", sn, addr);
                            stream.set_nonblocking(true)?;
                            socket.client = Some(stream);
                            self.raise_sn_ir(sn, SocketInterrupt::CON_MASK);
                            self.sim_set_sn_sr(sn, SocketStatus::Established);
                        }
                        Err(e) => match e.kind() {
                            io::ErrorKind::WouldBlock => {}
                            _ => return Err(e),
                        },
                    }
                }
            }
            None => {}
        };
        Ok(())
    }

    fn common_reg_rd(&self, addr: u16) -> u8 {
        let decoded = Reg::try_from(addr);

        let ret: u8 = match decoded {
            Ok(Reg::MR) => self.regs.mr,
            Ok(Reg::GAR0) => self.regs.gar.octets[0],
            Ok(Reg::GAR1) => self.regs.gar.octets[1],
            Ok(Reg::GAR2) => self.regs.gar.octets[2],
            Ok(Reg::GAR3) => self.regs.gar.octets[3],
            Ok(Reg::SUBR0) => self.regs.subr.octets[0],
            Ok(Reg::SUBR1) => self.regs.subr.octets[1],
            Ok(Reg::SUBR2) => self.regs.subr.octets[2],
            Ok(Reg::SUBR3) => self.regs.subr.octets[3],
            Ok(Reg::SHAR0) => self.regs.shar.octets[0],
            Ok(Reg::SHAR1) => self.regs.shar.octets[1],
            Ok(Reg::SHAR2) => self.regs.shar.octets[2],
            Ok(Reg::SHAR3) => self.regs.shar.octets[3],
            Ok(Reg::SHAR4) => self.regs.shar.octets[4],
            Ok(Reg::SHAR5) => self.regs.shar.octets[5],
            Ok(Reg::SIPR0) => self.regs.sipr.octets[0],
            Ok(Reg::SIPR1) => self.regs.sipr.octets[1],
            Ok(Reg::SIPR2) => self.regs.sipr.octets[2],
            Ok(Reg::SIPR3) => self.regs.sipr.octets[3],
            Ok(Reg::INTLEVEL0) => self.regs.intlevel.to_be_bytes()[0],
            Ok(Reg::INTLEVEL1) => self.regs.intlevel.to_be_bytes()[1],
            Ok(Reg::IR) => self.regs.ir,
            Ok(Reg::IMR) => self.regs.imr,
            Ok(Reg::SIR) => self.regs.sir,
            Ok(Reg::SIMR) => self.regs.simr,
            Ok(Reg::RTR0) => self.regs.rtr.to_be_bytes()[0],
            Ok(Reg::RTR1) => self.regs.rtr.to_be_bytes()[1],
            Ok(Reg::RCR) => self.regs.rcr,
            Ok(Reg::PTIMER) => self.regs.ptimer,
            Ok(Reg::PMAGIC) => self.regs.pmagic,
            Ok(Reg::PHAR0) => self.regs.phar.octets[0],
            Ok(Reg::PHAR1) => self.regs.phar.octets[1],
            Ok(Reg::PHAR2) => self.regs.phar.octets[2],
            Ok(Reg::PHAR3) => self.regs.phar.octets[3],
            Ok(Reg::PHAR4) => self.regs.phar.octets[4],
            Ok(Reg::PHAR5) => self.regs.phar.octets[5],
            Ok(Reg::PSID0) => self.regs.psid.to_be_bytes()[0],
            Ok(Reg::PSID1) => self.regs.psid.to_be_bytes()[1],
            Ok(Reg::PMRU0) => self.regs.pmru.to_be_bytes()[0],
            Ok(Reg::PMRU1) => self.regs.pmru.to_be_bytes()[1],
            Ok(Reg::UIPR0) => self.regs.uipr.octets[0],
            Ok(Reg::UIPR1) => self.regs.uipr.octets[1],
            Ok(Reg::UIPR2) => self.regs.uipr.octets[2],
            Ok(Reg::UIPR3) => self.regs.uipr.octets[3],
            Ok(Reg::UPORTR0) => self.regs.uportr.to_be_bytes()[0],
            Ok(Reg::UPORTR1) => self.regs.uportr.to_be_bytes()[1],
            Ok(Reg::PHYCFGR) => self.regs.phycfgr,
            Ok(Reg::VERSIONR) => self.regs.versionr,
            Err(_) => 0x00,
        };

        let (name, level): (String, log::Level) = match decoded {
            Ok(reg) => (format!("{:?}", reg), log::Level::Trace),
            Err(_) => (String::from("INVALID"), log::Level::Error),
        };
        log::log!(level, "[R] [COM] {:04X} -> {:02X} {}", addr, ret, name);

        ret
    }

    fn common_reg_wr(&mut self, addr: u16, byte: u8) -> io::Result<()> {
        let decoded = Reg::try_from(addr);

        match decoded {
            Ok(Reg::MR) => {
                self.regs.mr = byte;
                if byte & Mode::RST_MASK != 0 {
                    self.reset()
                }
                let mode: Mode = byte.into();
                if mode.wol_enabled() != Mode::default().wol_enabled() {
                    log::warn!("[W] [COM] MR wake on lan bit unimplemented");
                }
                if mode.pb_enabled() != Mode::default().pb_enabled() {
                    log::warn!("[W] [COM] MR ping block bit unimplemented");
                }
                if mode.pppoe_enabled() != Mode::default().pppoe_enabled() {
                    log::warn!("[W] [COM] MR PPPoE bit unimplemented");
                }
                if mode.farp_enabled() != Mode::default().farp_enabled() {
                    log::warn!("[W] [COM] MR force ARP bit unimplemented");
                }
            }
            Ok(Reg::GAR0) => self.regs.gar.octets[0] = byte,
            Ok(Reg::GAR1) => self.regs.gar.octets[1] = byte,
            Ok(Reg::GAR2) => self.regs.gar.octets[2] = byte,
            Ok(Reg::GAR3) => self.regs.gar.octets[3] = byte,
            Ok(Reg::SUBR0) => self.regs.subr.octets[0] = byte,
            Ok(Reg::SUBR1) => self.regs.subr.octets[1] = byte,
            Ok(Reg::SUBR2) => self.regs.subr.octets[2] = byte,
            Ok(Reg::SUBR3) => self.regs.subr.octets[3] = byte,
            Ok(Reg::SHAR0) => self.regs.shar.octets[0] = byte,
            Ok(Reg::SHAR1) => self.regs.shar.octets[1] = byte,
            Ok(Reg::SHAR2) => self.regs.shar.octets[2] = byte,
            Ok(Reg::SHAR3) => self.regs.shar.octets[3] = byte,
            Ok(Reg::SHAR4) => self.regs.shar.octets[4] = byte,
            Ok(Reg::SHAR5) => self.regs.shar.octets[5] = byte,
            Ok(Reg::SIPR0) => self.regs.sipr.octets[0] = byte,
            Ok(Reg::SIPR1) => self.regs.sipr.octets[1] = byte,
            Ok(Reg::SIPR2) => self.regs.sipr.octets[2] = byte,
            Ok(Reg::SIPR3) => self.regs.sipr.octets[3] = byte,
            Ok(Reg::INTLEVEL0) => {
                self.regs.intlevel &= 0x00FF;
                self.regs.intlevel |= u16::from(byte) << 8;
            }
            Ok(Reg::INTLEVEL1) => {
                self.regs.intlevel &= 0xFF00;
                self.regs.intlevel |= u16::from(byte);
            }
            Ok(Reg::IR) => self.regs.ir = byte,
            Ok(Reg::IMR) => self.regs.imr = byte,
            Ok(Reg::SIR) => self.regs.sir = byte,
            Ok(Reg::SIMR) => self.regs.simr = byte,
            Ok(Reg::RTR0) => {
                self.regs.rtr &= 0x00FF;
                self.regs.rtr |= u16::from(byte) << 8;
            }
            Ok(Reg::RTR1) => {
                self.regs.rtr &= 0xFF00;
                self.regs.rtr |= u16::from(byte);
            }
            Ok(Reg::RCR) => self.regs.rcr = byte,
            Ok(Reg::PTIMER) => self.regs.ptimer = byte,
            Ok(Reg::PMAGIC) => self.regs.pmagic = byte,
            Ok(Reg::PHAR0) => self.regs.phar.octets[0] = byte,
            Ok(Reg::PHAR1) => self.regs.phar.octets[1] = byte,
            Ok(Reg::PHAR2) => self.regs.phar.octets[2] = byte,
            Ok(Reg::PHAR3) => self.regs.phar.octets[3] = byte,
            Ok(Reg::PHAR4) => self.regs.phar.octets[4] = byte,
            Ok(Reg::PHAR5) => self.regs.phar.octets[5] = byte,
            Ok(Reg::PSID0) => {
                self.regs.psid &= 0x00FF;
                self.regs.psid |= u16::from(byte) << 8;
            }
            Ok(Reg::PSID1) => {
                self.regs.psid &= 0xFF00;
                self.regs.psid |= u16::from(byte);
            }
            Ok(Reg::PMRU0) => {
                self.regs.pmru &= 0x00FF;
                self.regs.pmru |= u16::from(byte) << 8;
            }
            Ok(Reg::PMRU1) => {
                self.regs.pmru &= 0xFF00;
                self.regs.pmru |= u16::from(byte);
            }
            Ok(Reg::UIPR0) => (),
            Ok(Reg::UIPR1) => (),
            Ok(Reg::UIPR2) => (),
            Ok(Reg::UIPR3) => (),
            Ok(Reg::UPORTR0) => (),
            Ok(Reg::UPORTR1) => (),
            Ok(Reg::PHYCFGR) => self.regs.phycfgr = byte,
            Ok(Reg::VERSIONR) => (),
            Err(_) => (),
        };

        let (name, level): (String, log::Level) = match decoded {
            Ok(reg) => {
                if reg.is_ro() {
                    (format!("{:?} is read-only", reg), log::Level::Error)
                } else {
                    (format!("{:?}", reg), log::Level::Trace)
                }
            }
            Err(_) => (String::from("INVALID"), log::Level::Error),
        };

        log::log!(level, "[W] [COM] {:04X} <- {:02X} {}", addr, byte, name);

        Ok(())
    }

    fn socket_reg_rd(&mut self, addr: u16, sn: Sn) -> io::Result<u8> {
        self.check_socket(sn)?;
        let decoded = SnReg::try_from(addr);
        let socket: &Socket = self.socket(sn);

        let ret: u8 = match decoded {
            Ok(SnReg::MR) => socket.regs.mr,
            Ok(SnReg::CR) => socket.regs.cr,
            Ok(SnReg::IR) => socket.regs.ir.into(),
            Ok(SnReg::SR) => socket.regs.sr.into(),
            Ok(SnReg::PORT0) => socket.regs.port.to_be_bytes()[0],
            Ok(SnReg::PORT1) => socket.regs.port.to_be_bytes()[1],
            Ok(SnReg::DHAR0) => socket.regs.dhar.octets[0],
            Ok(SnReg::DHAR1) => socket.regs.dhar.octets[1],
            Ok(SnReg::DHAR2) => socket.regs.dhar.octets[2],
            Ok(SnReg::DHAR3) => socket.regs.dhar.octets[3],
            Ok(SnReg::DHAR4) => socket.regs.dhar.octets[4],
            Ok(SnReg::DHAR5) => socket.regs.dhar.octets[5],
            Ok(SnReg::DIPR0) => socket.regs.dipr.octets[0],
            Ok(SnReg::DIPR1) => socket.regs.dipr.octets[1],
            Ok(SnReg::DIPR2) => socket.regs.dipr.octets[2],
            Ok(SnReg::DIPR3) => socket.regs.dipr.octets[3],
            Ok(SnReg::DPORT0) => socket.regs.dport.to_be_bytes()[0],
            Ok(SnReg::DPORT1) => socket.regs.dport.to_be_bytes()[1],
            Ok(SnReg::MSSR0) => socket.regs.mssr.to_be_bytes()[0],
            Ok(SnReg::MSSR1) => socket.regs.mssr.to_be_bytes()[1],
            Ok(SnReg::TOS) => socket.regs.tos,
            Ok(SnReg::TTL) => socket.regs.ttl,
            Ok(SnReg::RXBUF_SIZE) => socket.regs.rxbuf_size.into(),
            Ok(SnReg::TXBUF_SIZE) => socket.regs.txbuf_size.into(),
            Ok(SnReg::TX_FSR0) => socket.regs.tx_fsr.to_be_bytes()[0],
            Ok(SnReg::TX_FSR1) => socket.regs.tx_fsr.to_be_bytes()[1],
            Ok(SnReg::TX_RD0) => socket.regs.tx_rd.to_be_bytes()[0],
            Ok(SnReg::TX_RD1) => socket.regs.tx_rd.to_be_bytes()[1],
            Ok(SnReg::TX_WR0) => socket.regs.tx_wr.to_be_bytes()[0],
            Ok(SnReg::TX_WR1) => socket.regs.tx_wr.to_be_bytes()[1],
            Ok(SnReg::RX_RSR0) => socket.regs.rx_rsr.to_be_bytes()[0],
            Ok(SnReg::RX_RSR1) => socket.regs.rx_rsr.to_be_bytes()[1],
            Ok(SnReg::RX_RD0) => socket.regs.rx_rd.to_be_bytes()[0],
            Ok(SnReg::RX_RD1) => socket.regs.rx_rd.to_be_bytes()[1],
            Ok(SnReg::RX_WR0) => socket.regs.rx_wr.to_be_bytes()[0],
            Ok(SnReg::RX_WR1) => socket.regs.rx_wr.to_be_bytes()[1],
            Ok(SnReg::IMR) => socket.regs.imr,
            Ok(SnReg::FRAG0) => socket.regs.frag.to_be_bytes()[0],
            Ok(SnReg::FRAG1) => socket.regs.frag.to_be_bytes()[1],
            Ok(SnReg::KPALVTR) => socket.regs.kpalvtr,
            Err(_) => 0x00,
        };

        let (name, level): (String, log::Level) = match decoded {
            Ok(reg) => (format!("{:?}", reg), log::Level::Trace),
            Err(_) => (String::from("INVALID"), log::Level::Error),
        };
        log::log!(level, "[R] [{:?}] {:04X} -> {:02X} {}", sn, addr, ret, name);

        Ok(ret)
    }

    fn socket_reg_wr(&mut self, addr: u16, byte: u8, sn: Sn) -> io::Result<()> {
        let decoded = SnReg::try_from(addr);
        let socket: &mut Socket = self.socket_mut(sn);

        match decoded {
            Ok(SnReg::MR) => {
                socket.regs.mr = byte;
            }
            Ok(SnReg::CR) => match SocketCommand::try_from(byte) {
                Ok(SocketCommand::Open) => self.socket_cmd_open(sn)?,
                Ok(SocketCommand::Connect) => self.socket_cmd_connect(sn)?,
                Ok(SocketCommand::Close) => self.socket_cmd_close(sn),
                Ok(SocketCommand::Send) => self.socket_cmd_send(sn)?,
                Ok(SocketCommand::Recv) => self.socket_cmd_recv(sn)?,
                Ok(SocketCommand::Listen) => self.socket_cmd_listen(sn)?,
                cmd => unimplemented!("[W] [{:?}] command {:?}", sn, cmd),
            },
            Ok(SnReg::IR) => {
                let ir: SocketInterrupt = byte.into();

                if socket.regs.ir.con_raised() & ir.con_raised() {
                    log::debug!("[{:?}] clearing CON_MASK interrupt", sn);
                    socket.regs.ir = socket.regs.ir.clear_con();
                }
                if socket.regs.ir.discon_raised() & ir.discon_raised() {
                    log::debug!("[{:?}] clearing DISCON_MASK interrupt", sn);
                    socket.regs.ir = socket.regs.ir.clear_discon();
                }
                if socket.regs.ir.recv_raised() & ir.recv_raised() {
                    log::debug!("[{:?}] clearing RECV_MASK interrupt", sn);
                    socket.regs.ir = socket.regs.ir.clear_recv();
                }
                if socket.regs.ir.timeout_raised() & ir.timeout_raised() {
                    log::debug!("[{:?}] clearing TIMEOUT_MASK interrupt", sn);
                    socket.regs.ir = socket.regs.ir.clear_timeout();
                }
                if socket.regs.ir.sendok_raised() & ir.sendok_raised() {
                    log::debug!("[{:?}] clearing SENDOK_MASK interrupt", sn);
                    socket.regs.ir = socket.regs.ir.clear_sendok();
                }

                if u8::from(socket.regs.ir) & socket.regs.imr & 0x1F == 0 {
                    self.regs.sir &= !sn.bitmask();
                }
            }
            Ok(SnReg::SR) => (),
            Ok(SnReg::PORT0) => {
                socket.regs.port &= 0x00FF;
                socket.regs.port |= u16::from(byte) << 8;
            }
            Ok(SnReg::PORT1) => {
                socket.regs.port &= 0xFF00;
                socket.regs.port |= u16::from(byte);
            }
            Ok(SnReg::DHAR0) => socket.regs.dhar.octets[0] = byte,
            Ok(SnReg::DHAR1) => socket.regs.dhar.octets[1] = byte,
            Ok(SnReg::DHAR2) => socket.regs.dhar.octets[2] = byte,
            Ok(SnReg::DHAR3) => socket.regs.dhar.octets[3] = byte,
            Ok(SnReg::DHAR4) => socket.regs.dhar.octets[4] = byte,
            Ok(SnReg::DHAR5) => socket.regs.dhar.octets[5] = byte,
            Ok(SnReg::DIPR0) => socket.regs.dipr.octets[0] = byte,
            Ok(SnReg::DIPR1) => socket.regs.dipr.octets[1] = byte,
            Ok(SnReg::DIPR2) => socket.regs.dipr.octets[2] = byte,
            Ok(SnReg::DIPR3) => socket.regs.dipr.octets[3] = byte,
            Ok(SnReg::DPORT0) => {
                socket.regs.dport &= 0x00FF;
                socket.regs.dport |= u16::from(byte) << 8;
            }
            Ok(SnReg::DPORT1) => {
                socket.regs.dport &= 0xFF00;
                socket.regs.dport |= u16::from(byte);
            }
            Ok(SnReg::MSSR0) => todo!(),
            Ok(SnReg::MSSR1) => todo!(),
            Ok(SnReg::TOS) => todo!(),
            Ok(SnReg::TTL) => todo!(),
            Ok(SnReg::RXBUF_SIZE) => todo!(),
            Ok(SnReg::TXBUF_SIZE) => todo!(),
            Ok(SnReg::TX_FSR0) => (),
            Ok(SnReg::TX_FSR1) => (),
            Ok(SnReg::TX_RD0) => (),
            Ok(SnReg::TX_RD1) => (),
            Ok(SnReg::TX_WR0) => {
                socket.regs.tx_wr &= 0x00FF;
                socket.regs.tx_wr |= u16::from(byte) << 8;
            }
            Ok(SnReg::TX_WR1) => {
                socket.regs.tx_wr &= 0xFF00;
                socket.regs.tx_wr |= u16::from(byte);
            }
            Ok(SnReg::RX_RSR0) => (),
            Ok(SnReg::RX_RSR1) => (),
            Ok(SnReg::RX_RD0) => {
                socket.regs.rx_rd &= 0x00FF;
                socket.regs.rx_rd |= u16::from(byte) << 8;
            }
            Ok(SnReg::RX_RD1) => {
                socket.regs.rx_rd &= 0xFF00;
                socket.regs.rx_rd |= u16::from(byte);
            }
            Ok(SnReg::RX_WR0) => todo!(),
            Ok(SnReg::RX_WR1) => todo!(),
            Ok(SnReg::IMR) => todo!(),
            Ok(SnReg::FRAG0) => todo!(),
            Ok(SnReg::FRAG1) => todo!(),
            Ok(SnReg::KPALVTR) => todo!(),
            Err(_) => (),
        }

        let (name, level): (String, log::Level) = match decoded {
            Ok(reg) => {
                if reg.is_ro() {
                    (format!("{:?} is read-only", reg), log::Level::Error)
                } else {
                    (format!("{:?}", reg), log::Level::Trace)
                }
            }
            Err(_) => (String::from("INVALID"), log::Level::Error),
        };

        log::log!(
            level,
            "[W] [{:?}] {:04X} <- {:02X} {}",
            sn,
            addr,
            byte,
            name
        );

        Ok(())
    }
}

impl Default for W5500 {
    fn default() -> Self {
        Self {
            regs: CommonRegs::RESET,
            sn: Default::default(),
        }
    }
}

impl Registers for W5500 {
    type Error = std::io::Error;

    fn read(&mut self, addr: u16, block: u8, data: &mut [u8]) -> Result<(), Self::Error> {
        let mut addr: u16 = addr;

        match block_type(block) {
            BlockType::Common => {
                data.iter_mut().for_each(|byte| {
                    *byte = self.common_reg_rd(addr);
                    addr = addr.wrapping_add(1);
                });
                Ok(())
            }
            BlockType::Socket(sn) => {
                for byte in data.iter_mut() {
                    *byte = self.socket_reg_rd(addr, sn)?;
                    addr = addr.wrapping_add(1);
                }
                Ok(())
            }
            BlockType::Rx(sn) => {
                data.iter_mut().for_each(|byte| {
                    *byte = self.sn[usize::from(sn)].rx_buf[usize::from(addr)];
                    log::trace!("[R] [RXB] {:04X} -> {:02X}", addr, *byte);
                    addr = addr.wrapping_add(1);
                });
                Ok(())
            }
            BlockType::Tx(sn) => {
                data.iter_mut().for_each(|byte| {
                    *byte = self.sn[usize::from(sn)].tx_buf[usize::from(addr)];
                    log::trace!("[R] [TXB] {:04X} -> {:02X}", addr, *byte);
                    addr = addr.wrapping_add(1);
                });
                Ok(())
            }
        }
    }

    /// Write to the W5500.
    fn write(&mut self, addr: u16, block: u8, data: &[u8]) -> Result<(), Self::Error> {
        let mut addr: u16 = addr;

        match block_type(block) {
            BlockType::Common => {
                for byte in data {
                    self.common_reg_wr(addr, *byte)?;
                    addr = addr.wrapping_add(1);
                }
                Ok(())
            }
            BlockType::Socket(sn) => {
                for byte in data {
                    self.socket_reg_wr(addr, *byte, sn)?;
                    addr = addr.wrapping_add(1);
                }
                Ok(())
            }
            BlockType::Rx(sn) => {
                data.iter().for_each(|byte| {
                    log::trace!("[W] [RXB] {:04X} <- {:02X}", addr, *byte);
                    self.sn[usize::from(sn)].rx_buf[usize::from(addr)] = *byte;
                    addr = addr.wrapping_add(1);
                });
                Ok(())
            }
            BlockType::Tx(sn) => {
                data.iter().for_each(|byte| {
                    log::trace!("[W] [TXB] {:04X} <- {:02X}", addr, *byte);
                    self.sn[usize::from(sn)].tx_buf[usize::from(addr)] = *byte;
                    addr = addr.wrapping_add(1);
                });
                Ok(())
            }
        }
    }
}
