//! Register simulation for the [Wiznet W5500] internet offload chip.
//!
//! # Example
//!
//! See the [`w5500-hl`] crate for examples.
//!
//! # Notes
//!
//! This is in an early alpha state, there are many todos throughout the code.
//! Bug reports will not be accepted until this reaches `0.1.0`.
//! Pull requests are always welcome.
//!
//! At the moment this only does really basic UDP and TCP streams.
//! TCP listeners have not yet been implemented.
//!
//! It is not possible to fully simulate the W5500 without spending a silly
//! ammount of time on this crate.
//! This is a best-effort implementation to aid in development of application
//! code.
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
//!     * Listen (TCP listeners not yet implemented)
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
//! ## Assumptions
//!
//! * Your PC is connected to a network, and has a valid IPv4 address.
//! * You are not using the `read` and `write` methods directly.
//!
//! [Wiznet W5500]: https://www.wiznet.io/product-item/w5500/
//! [`w5500-hl`]: https://crates.io/crates/w5500-hl
#![doc(html_root_url = "https://docs.rs/w5500-regsim/0.1.0-alpha.1")]

mod regmap;

use std::{
    cmp::min,
    convert::TryFrom,
    io::{self, Read, Write},
    net::{SocketAddrV4, TcpStream, UdpSocket},
};

pub use w5500_ll::{self, Registers};

use w5500_ll::{
    net::{Eui48Addr, Ipv4Addr},
    reg, BufferSize, Mode, Socket, SocketCommand, SocketInterrupt, SocketStatus,
    COMMON_BLOCK_OFFSET, SOCKETS, VERSION,
};

// Socket spacing between blocks.
const SOCKET_SPACING: u8 = 0x04;

const COMMON_REGS_SIZE: usize = 0x40;
const SOCKET_REGS_SIZE: usize = 0x30;
const NUM_SOCKETS: usize = SOCKETS.len();
const DEFAULT_BUF_SIZE: usize = BufferSize::KB2.size_in_bytes();

const RO_COMMON_REGS: &[u16] = &[reg::UIPR, reg::UPORTR, reg::VERSIONR];
const RO_SOCKET_REGS: &[u16] = &[
    0x14,
    0x17,
    0x18,
    0x19,
    0x1A,
    0x1B,
    0x1C,
    0x1D,
    reg::SN_SR,
    reg::SN_TX_FSR,
    reg::SN_TX_RD,
    reg::SN_RX_RSR,
];

enum SocketType {
    Udp(UdpSocket),
    // TcpListener(TcpListener),
    TcpStream(TcpStream),
}

#[derive(PartialEq, Eq)]
enum BlockType {
    Common,
    Socket(Socket),
    Tx(Socket),
    Rx(Socket),
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
                write!(f, "SN{}", u8::from(*n))
            }
        }
    }
}

impl BlockType {
    pub fn is_socket_buf(&self) -> bool {
        matches!(self, BlockType::Tx(_) | BlockType::Rx(_))
    }

    pub fn addr_fmt(&self, addr: u16) -> String {
        match self {
            BlockType::Common => {
                format!("[0x{:02X}] ({})", addr, regmap::common_reg_name(&addr))
            }
            BlockType::Socket(_) => {
                format!("[0x{:02X}] ({})", addr, regmap::socket_reg_name(&addr))
            }
            BlockType::Tx(_) | BlockType::Rx(_) => {
                format!("[0x{:04X}]", addr)
            }
        }
    }
}

fn block_type(block: u8) -> BlockType {
    if block == 0 {
        BlockType::Common
    } else {
        let sn_val: u8 = block / SOCKET_SPACING;
        let sn: Socket = Socket::try_from(sn_val)
            .unwrap_or_else(|_| panic!("Invalid block address: 0x{:02X}", block));
        match block - (sn_val * SOCKET_SPACING) {
            1 => BlockType::Socket(sn),
            2 => BlockType::Tx(sn),
            3 => BlockType::Rx(sn),
            _ => panic!("Invalid block address: 0x{:02X}", block),
        }
    }
}

/// Simulated W5500.
pub struct W5500 {
    common_regs: [u8; COMMON_REGS_SIZE],
    socket_regs: [[u8; SOCKET_REGS_SIZE]; NUM_SOCKETS],
    tx_buf: [Vec<u8>; NUM_SOCKETS],
    rx_buf: [Vec<u8>; NUM_SOCKETS],
    sockets: [Option<SocketType>; NUM_SOCKETS],
}

impl W5500 {
    /// Create a new simulated W5500.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_regsim::W5500;
    ///
    /// let w5500 = W5500::new();
    /// ```
    pub fn new() -> W5500 {
        let mut device = W5500 {
            common_regs: [0; COMMON_REGS_SIZE],
            socket_regs: [[0; SOCKET_REGS_SIZE]; NUM_SOCKETS],
            tx_buf: [
                vec![0; DEFAULT_BUF_SIZE],
                vec![0; DEFAULT_BUF_SIZE],
                vec![0; DEFAULT_BUF_SIZE],
                vec![0; DEFAULT_BUF_SIZE],
                vec![0; DEFAULT_BUF_SIZE],
                vec![0; DEFAULT_BUF_SIZE],
                vec![0; DEFAULT_BUF_SIZE],
                vec![0; DEFAULT_BUF_SIZE],
            ],
            rx_buf: [
                vec![0; DEFAULT_BUF_SIZE],
                vec![0; DEFAULT_BUF_SIZE],
                vec![0; DEFAULT_BUF_SIZE],
                vec![0; DEFAULT_BUF_SIZE],
                vec![0; DEFAULT_BUF_SIZE],
                vec![0; DEFAULT_BUF_SIZE],
                vec![0; DEFAULT_BUF_SIZE],
                vec![0; DEFAULT_BUF_SIZE],
            ],
            sockets: [None, None, None, None, None, None, None, None],
        };
        device.reset();
        device
    }

    fn reset(&mut self) {
        self.common_regs = [0; COMMON_REGS_SIZE];
        self.common_regs[usize::from(reg::RTR)] = 0x07;
        self.common_regs[usize::from(reg::RTR + 1)] = 0xD0;
        self.common_regs[usize::from(reg::RCR)] = 0x08;
        self.common_regs[usize::from(reg::PTIMER + 1)] = 0x1C;
        self.common_regs[usize::from(reg::PMAGIC + 1)] = 0x1D;
        self.common_regs[usize::from(reg::PMRU)] = 0xFF;
        self.common_regs[usize::from(reg::PMRU + 1)] = 0xFF;
        self.common_regs[usize::from(reg::PHYCFGR)] = 0b10111111;
        self.common_regs[usize::from(reg::VERSIONR)] = VERSION;

        self.socket_regs = [[0; SOCKET_REGS_SIZE]; NUM_SOCKETS];
        for regs in self.socket_regs.iter_mut() {
            regs[usize::from(reg::SN_DHAR)] = 0xFF;
            regs[usize::from(reg::SN_DHAR + 1)] = 0xFF;
            regs[usize::from(reg::SN_DHAR + 2)] = 0xFF;
            regs[usize::from(reg::SN_DHAR + 3)] = 0xFF;
            regs[usize::from(reg::SN_TTL)] = 0x80;
            regs[usize::from(reg::SN_RXBUF_SIZE)] = 0x02;
            regs[usize::from(reg::SN_TXBUF_SIZE)] = 0x02;
            regs[usize::from(reg::SN_TX_FSR)] = 0x08;
            regs[usize::from(reg::SN_IMR)] = 0xFF;
            regs[usize::from(reg::SN_FRAG)] = 0x40;
        }

        self.tx_buf = [
            vec![0; DEFAULT_BUF_SIZE],
            vec![0; DEFAULT_BUF_SIZE],
            vec![0; DEFAULT_BUF_SIZE],
            vec![0; DEFAULT_BUF_SIZE],
            vec![0; DEFAULT_BUF_SIZE],
            vec![0; DEFAULT_BUF_SIZE],
            vec![0; DEFAULT_BUF_SIZE],
            vec![0; DEFAULT_BUF_SIZE],
        ];
        self.rx_buf = [
            vec![0; DEFAULT_BUF_SIZE],
            vec![0; DEFAULT_BUF_SIZE],
            vec![0; DEFAULT_BUF_SIZE],
            vec![0; DEFAULT_BUF_SIZE],
            vec![0; DEFAULT_BUF_SIZE],
            vec![0; DEFAULT_BUF_SIZE],
            vec![0; DEFAULT_BUF_SIZE],
            vec![0; DEFAULT_BUF_SIZE],
        ];
    }

    fn buf_from_block(&mut self, block: u8) -> &mut [u8] {
        match block_type(block) {
            BlockType::Common => &mut self.common_regs,
            BlockType::Socket(sn) => &mut self.socket_regs[usize::from(sn)],
            BlockType::Tx(sn) => &mut self.tx_buf[usize::from(sn)],
            BlockType::Rx(sn) => &mut self.rx_buf[usize::from(sn)],
        }
    }

    fn set_sn_sr(&mut self, socket: Socket, state: SocketStatus) {
        self.socket_regs[usize::from(socket)][usize::from(reg::SN_SR)] = state.into()
    }

    fn std_sn_dest(&mut self, socket: Socket) -> std::net::SocketAddrV4 {
        let ip = self.sn_dipr(socket).unwrap();
        let port = self.sn_dport(socket).unwrap();
        std::net::SocketAddrV4::new(std::net::Ipv4Addr::from(ip.octets), port)
    }

    fn socket_cmd_open(&mut self, socket: Socket) -> io::Result<()> {
        match self
            .sn_mr(socket)
            .unwrap()
            .protocol()
            .expect("invalid protocol bits")
        {
            w5500_ll::Protocol::Closed => {
                panic!(
                    "You should set sn_mr with the protocol before sending {:?} to {:?}",
                    SocketCommand::Open,
                    socket
                )
            }
            w5500_ll::Protocol::Tcp => {
                log::info!("[{:?}] Setting mode to TCP", socket);
                self.set_sn_sr(socket, SocketStatus::Init);
                self.sockets[usize::from(socket)] = None;
            }
            w5500_ll::Protocol::Udp => {
                let port = self.sn_dport(socket).unwrap();
                let local = SocketAddrV4::new(std::net::Ipv4Addr::new(0, 0, 0, 0), port);
                log::info!("[{:?}] Binding UDP socket to {}", socket, local);
                self.set_sn_sr(socket, SocketStatus::Udp);
                match UdpSocket::bind(local) {
                    Ok(udp_socket) => {
                        log::info!("[{:?}] Successfully bound to {}", socket, local);
                        udp_socket.set_nonblocking(true)?;
                        self.sockets[usize::from(socket)] = Some(SocketType::Udp(udp_socket));
                    }
                    Err(e) => {
                        log::warn!("[{:?}] Failed to bind socket {}: {}", socket, local, e);
                        self.set_sn_sr(socket, SocketStatus::Closed);
                    }
                }
            }
            w5500_ll::Protocol::Macraw => {
                if socket == Socket::Socket0 {
                    panic!("MACRAW is not implemented")
                } else {
                    panic!("MACRAW can only be used on Socket0, not {:?}", socket)
                }
            }
        }
        Ok(())
    }

    fn socket_cmd_connect(&mut self, socket: Socket) -> io::Result<()> {
        if self
            .sn_sr(socket)
            .unwrap()
            .expect("Invalid socket status bits")
            != SocketStatus::Init
        {
            panic!(
                "You should only send {:?} after initializing {:?} as TCP",
                SocketCommand::Connect,
                socket
            )
        }
        let addr = self.std_sn_dest(socket);
        log::info!("[{:?}] Opening a TCP stream to {}", socket, addr);

        match TcpStream::connect(self.std_sn_dest(socket)) {
            Ok(stream) => {
                log::info!("[{:?}] Established TCP connection with {}", socket, addr);
                stream.set_nonblocking(true)?;
                self.sockets[usize::from(socket)] = Some(SocketType::TcpStream(stream));
                self.raise_sn_ir(socket, SocketInterrupt::CON_MASK);
                self.set_sn_sr(socket, SocketStatus::Established);
            }
            Err(e) => {
                log::warn!("[{:?}] TCP stream to {} failed: {}", socket, addr, e);
                self.socket_regs[usize::from(socket)][usize::from(reg::SN_IR)] =
                    SocketInterrupt::DISCON_MASK;
                self.set_sn_sr(socket, SocketStatus::Closed);
            }
        }

        Ok(())
    }

    fn socket_cmd_close(&mut self, socket: Socket) {
        self.sockets[usize::from(socket)] = None;
        self.set_sn_sr(socket, SocketStatus::Closed);
    }

    fn socket_cmd_send(&mut self, socket: Socket) -> io::Result<()> {
        let tail: usize = self.sn_tx_rd(socket)?.into();
        let head: usize = self.sn_tx_wr(socket)?.into();
        if head == tail {
            panic!(
                "Got command {:?} on {:?} without anything to send",
                SocketCommand::Send,
                socket,
            );
        }
        let size: usize = if head >= tail {
            head - tail
        } else {
            usize::from(u16::MAX) + head - tail
        };

        log::debug!("[{:?}] sn_tx_rd=0x{:04X}", socket, tail);
        log::debug!("[{:?}] sn_tx_wr=0x{:04X}", socket, head);
        log::debug!("[{:?}] size=0x{:04X}", socket, size);

        debug_assert!(
            size <= self.priv_sn_txbuf_size(socket),
            "Send data size exceeds buffer size"
        );

        let mut local_tx_buf: Vec<u8> = Vec::with_capacity(size);
        let buf = &self.tx_buf[usize::from(socket)];

        if head >= tail {
            for buffer_adr in tail..head {
                let buf_idx = buffer_adr % buf.len();
                local_tx_buf.push(buf[buf_idx]);
            }
        } else {
            for buffer_adr in tail..usize::from(u16::MAX) {
                let buf_idx = buffer_adr % buf.len();
                local_tx_buf.push(buf[buf_idx]);
            }
            for buffer_adr in 0..head {
                let buf_idx = buffer_adr % buf.len();
                local_tx_buf.push(buf[buf_idx]);
            }
        }

        assert!(!local_tx_buf.is_empty());

        let dest = self.std_sn_dest(socket);

        match (&mut self.sockets[usize::from(socket)]).as_mut() {
            Some(SocketType::TcpStream(ref mut stream)) => {
                stream.write_all(&local_tx_buf)?;
            }
            Some(SocketType::Udp(ref mut udp)) => {
                log::info!("[{:?}] sending to {}", socket, dest);
                let num: usize = udp.send_to(&local_tx_buf, &dest)?;
                assert_eq!(num, local_tx_buf.len());
            }
            None => {
                panic!("Unable to send data, {:?} is closed", socket)
            }
        }

        Ok(())
    }

    fn socket_cmd_recv(&mut self, socket: Socket) -> io::Result<()> {
        log::error!("[{:?}] TODO RECV", socket);
        Ok(())
    }

    /// Private RSR accessor to prevent logging "internal" IO.
    fn priv_sn_rx_rsr(&self, socket: Socket) -> u16 {
        u16::from_be_bytes([
            self.socket_regs[usize::from(socket)][usize::from(reg::SN_RX_RSR)],
            self.socket_regs[usize::from(socket)][usize::from(reg::SN_RX_RSR + 1)],
        ])
    }

    /// Private RSR accessor to prevent logging "internal" IO.
    fn priv_sn_rx_wr(&self, socket: Socket) -> u16 {
        u16::from_be_bytes([
            self.socket_regs[usize::from(socket)][usize::from(reg::SN_RX_WR)],
            self.socket_regs[usize::from(socket)][usize::from(reg::SN_RX_WR + 1)],
        ])
    }

    fn set_sn_rx_rsr(&mut self, socket: Socket, rsr: u16) {
        let current_rsr = self.priv_sn_rx_rsr(socket);
        log::debug!(
            "[{:?}] Updating SN_RX_RSR 0x{:04X} -> 0x{:04X}",
            socket,
            current_rsr,
            rsr
        );
        self.socket_regs[usize::from(socket)][usize::from(reg::SN_RX_RSR)] = rsr.to_be_bytes()[0];
        self.socket_regs[usize::from(socket)][usize::from(reg::SN_RX_RSR + 1)] =
            rsr.to_be_bytes()[1];
    }

    fn set_sn_rx_wr(&mut self, socket: Socket, wr: u16) {
        let current_wr = self.priv_sn_rx_wr(socket);
        log::debug!(
            "[{:?}] Updating SN_RX_WR 0x{:04X} -> 0x{:04X}",
            socket,
            current_wr,
            wr
        );
        self.socket_regs[usize::from(socket)][usize::from(reg::SN_RX_WR)] = wr.to_be_bytes()[0];
        self.socket_regs[usize::from(socket)][usize::from(reg::SN_RX_WR + 1)] = wr.to_be_bytes()[1];
    }

    fn write_rx_buf(&mut self, socket: Socket, data: &[u8]) {
        let mut rsr = self.priv_sn_rx_rsr(socket);
        let mut wr = self.priv_sn_rx_wr(socket);

        let buf = self.buf_from_block(socket.rx_block());
        for byte in data.iter() {
            let buf_idx = usize::from(wr) % buf.len();
            buf[buf_idx] = *byte;
            wr = wr.wrapping_add(1);
            rsr = rsr.saturating_add(1);
        }

        rsr = min(rsr, u16::try_from(buf.len()).unwrap_or(u16::MAX));

        self.set_sn_rx_rsr(socket, rsr);
        self.set_sn_rx_wr(socket, wr);
    }

    fn priv_sn_rxbuf_size(&mut self, socket: Socket) -> usize {
        usize::from(self.socket_regs[usize::from(socket)][usize::from(reg::SN_RXBUF_SIZE)]) * 1024
    }

    fn priv_sn_txbuf_size(&mut self, socket: Socket) -> usize {
        usize::from(self.socket_regs[usize::from(socket)][usize::from(reg::SN_TXBUF_SIZE)]) * 1024
    }

    fn raise_sn_ir(&mut self, socket: Socket, int: u8) {
        self.socket_regs[usize::from(socket)][usize::from(reg::SN_IR)] |= int;
    }

    fn handle_socket(&mut self, socket: Socket) -> io::Result<()> {
        let bufsize = self.priv_sn_rxbuf_size(socket);
        let mut buf = vec![0; bufsize];

        match self.sockets[usize::from(socket)] {
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
                    log::info!(
                        "[{:?}] recv datagram of len {} from {}",
                        socket,
                        num,
                        origin
                    );
                    let numu16 = u16::try_from(num).unwrap();
                    // write out the header
                    self.write_rx_buf(socket, &origin.ip().octets());
                    self.write_rx_buf(socket, &origin.port().to_be_bytes());
                    self.write_rx_buf(socket, &numu16.to_be_bytes());
                    // write the rest of the data
                    self.write_rx_buf(socket, &buf[..num]);
                    self.raise_sn_ir(socket, SocketInterrupt::RECV_MASK);
                    log::warn!("TODO: shorten buf by 8 for a UDP socket");
                }
                Err(e) => match e.kind() {
                    io::ErrorKind::WouldBlock => {}
                    _ => return Err(e),
                },
            },
            Some(SocketType::TcpStream(ref mut tcp)) => match tcp.read(&mut buf) {
                Ok(num) => {
                    log::info!("[{:?}] recv {} bytes", socket, num);
                    self.write_rx_buf(socket, &buf[..num]);
                    self.raise_sn_ir(socket, SocketInterrupt::RECV_MASK);
                }
                Err(e) => match e.kind() {
                    io::ErrorKind::WouldBlock => {}
                    _ => return Err(e),
                },
            },

            None => {}
        };
        Ok(())
    }
}

impl Default for W5500 {
    fn default() -> Self {
        W5500::new()
    }
}

impl Registers for W5500 {
    type Error = std::io::Error;

    fn set_mr(&mut self, mode: Mode) -> io::Result<()> {
        if u8::from(mode) & Mode::RST_MASK != 0 {
            self.reset()
        }
        if mode.wol_enabled() != Mode::default().wol_enabled() {
            log::warn!("set_mr wake on lan bit not implemented");
        }
        if mode.pb_enabled() != Mode::default().pb_enabled() {
            log::warn!("set_mr ping block bit not implemented");
        }
        if mode.pppoe_enabled() != Mode::default().pppoe_enabled() {
            log::warn!("set_mr PPPoE bit not implemented");
        }
        if mode.farp_enabled() != Mode::default().farp_enabled() {
            log::warn!("set_mr force ARP bit not implemented");
        }
        Ok(())
    }

    fn gar(&mut self) -> io::Result<Ipv4Addr> {
        todo!()
    }

    fn set_gar(&mut self, gar: &Ipv4Addr) -> io::Result<()> {
        log::warn!("set_gar({}) does nothing", gar);
        Ok(())
    }

    fn subr(&mut self) -> io::Result<Ipv4Addr> {
        todo!()
    }

    fn set_subr(&mut self, subr: &Ipv4Addr) -> io::Result<()> {
        log::warn!("set_subr({}) does nothing", subr);
        Ok(())
    }

    fn shar(&mut self) -> io::Result<Eui48Addr> {
        todo!()
    }

    fn set_shar(&mut self, shar: &Eui48Addr) -> io::Result<()> {
        log::warn!("set_shar({}) does nothing", shar);
        Ok(())
    }

    fn sipr(&mut self) -> io::Result<Ipv4Addr> {
        todo!()
    }

    fn set_sipr(&mut self, sipr: &Ipv4Addr) -> io::Result<()> {
        log::warn!("set_sipr({}) does nothing", sipr);
        Ok(())
    }

    fn set_sn_cr(&mut self, socket: Socket, cmd: SocketCommand) -> io::Result<()> {
        match cmd {
            SocketCommand::Open => self.socket_cmd_open(socket)?,
            SocketCommand::Connect => self.socket_cmd_connect(socket)?,
            SocketCommand::Close => self.socket_cmd_close(socket),
            SocketCommand::Send => self.socket_cmd_send(socket)?,
            SocketCommand::Recv => self.socket_cmd_recv(socket)?,
            _ => unimplemented!("Socket command {:?} sent to {:?}", cmd, socket),
        }
        Ok(())
    }

    fn set_sn_tx_wr(&mut self, socket: Socket, ptr: u16) -> Result<(), Self::Error> {
        self.write(reg::SN_TX_WR, socket.block(), &ptr.to_be_bytes())?;
        // TODO: this should actually occur only on send.
        // decrement free size
        let mut sr = self.socket_regs[usize::from(socket)];
        let mut fsr = self.sn_tx_fsr(socket).unwrap();
        fsr = fsr.saturating_sub(ptr);
        let fsr_bytes = u16::to_be_bytes(fsr);
        sr[usize::from(reg::SN_TX_FSR)] = fsr_bytes[0];
        sr[usize::from(reg::SN_TX_FSR) + 1] = fsr_bytes[1];
        Ok(())
    }

    fn sn_ir(&mut self, socket: Socket) -> Result<SocketInterrupt, Self::Error> {
        self.handle_socket(socket)?;
        let mut reg: [u8; 1] = [0];
        self.read(reg::SN_IR, socket.block(), &mut reg)?;
        Ok(SocketInterrupt::from(reg[0]))
    }

    fn sn_rx_rsr(&mut self, socket: Socket) -> Result<u16, Self::Error> {
        self.handle_socket(socket)?;
        let mut reg: [u8; 2] = [0; 2];
        self.read(reg::SN_RX_RSR, socket.block(), &mut reg)?;
        Ok(u16::from_be_bytes(reg))
    }

    /// Read from the W5500.
    fn read(&mut self, address: u16, block: u8, data: &mut [u8]) -> Result<(), Self::Error> {
        let buf: &mut [u8] = self.buf_from_block(block);
        let address = usize::from(address);

        let block_type = block_type(block);

        if block_type.is_socket_buf() {
            for (offset, byte) in data.iter_mut().enumerate() {
                let buf_idx = (address + offset) % buf.len();
                *byte = buf[buf_idx];
                log::trace!(
                    "[R] [{}] {} 0x{:02X}",
                    block_type,
                    block_type.addr_fmt(u16::try_from(address + offset).unwrap()),
                    buf[buf_idx]
                );
            }
        } else {
            for (offset, byte) in data.iter_mut().enumerate() {
                *byte = buf[address + offset];
                log::trace!(
                    "[R] [{}] {} 0x{:02X}",
                    block_type,
                    block_type.addr_fmt(u16::try_from(address + offset).unwrap()),
                    buf[address + offset]
                );
            }
        }
        Ok(())
    }

    /// Write to the W5500.
    fn write(&mut self, address: u16, block: u8, data: &[u8]) -> Result<(), Self::Error> {
        let buf: &mut [u8] = self.buf_from_block(block);

        if block == COMMON_BLOCK_OFFSET && RO_COMMON_REGS.iter().any(|i| i == &address) {
            panic!(
                "Write to read only common register at address: 0x{:04X}",
                address
            );
        }

        if block % SOCKET_SPACING == 1 && RO_SOCKET_REGS.iter().any(|i| i == &address) {
            panic!(
                "Write to read only socket register at address: 0x{:04X}",
                address
            );
        }

        let block_type = block_type(block);

        if block_type.is_socket_buf() {
            for (offset, byte) in data.iter().enumerate() {
                let buf_idx = (usize::from(address) + offset) % buf.len();
                log::trace!(
                    "[W] [{}] {} 0x{:02X}",
                    block_type,
                    block_type.addr_fmt(u16::try_from(offset).unwrap() + address),
                    *byte
                );
                buf[buf_idx] = *byte;
            }
        } else {
            let address = usize::from(address);
            for (offset, byte) in data.iter().enumerate() {
                log::trace!(
                    "[W] [{}] {} 0x{:02X}",
                    block_type,
                    block_type.addr_fmt(u16::try_from(offset + address).unwrap()),
                    *byte
                );
                buf[address + offset] = *byte;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // smoke test for panic
    #[test]
    fn buf_from_block() {
        let mut w5500 = W5500::new();
        for socket in SOCKETS.iter() {
            w5500.buf_from_block(socket.block());
            w5500.buf_from_block(socket.tx_block());
            w5500.buf_from_block(socket.rx_block());
        }
    }
}
