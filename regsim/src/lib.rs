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
//! Bug reports will not be accepted until this reaches `0.1.0`.
//! Pull requests are always welcome.
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
//!
//! [Wiznet W5500]: https://www.wiznet.io/product-item/w5500/
//! [`std::net`]: https://doc.rust-lang.org/std/net/index.html
//! [`w5500-hl`]: https://crates.io/crates/w5500-hl
//! [`w5500_ll::Registers`]: https://docs.rs/w5500-ll/latest/w5500_ll/trait.Registers.html
#![doc(html_root_url = "https://docs.rs/w5500-regsim/0.1.0-alpha.2")]

mod regmap;

use std::{
    convert::TryFrom,
    io::{self, Read, Write},
    net::{SocketAddrV4, TcpListener, TcpStream, UdpSocket},
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
    TcpListener(TcpListener),
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
                write!(f, "RX{}", u8::from(*n))
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
    clients: [Option<TcpStream>; NUM_SOCKETS],
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
            clients: [None, None, None, None, None, None, None, None],
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

    /// Get the local buffer given block select bits.
    fn buf_from_block(&mut self, block: u8) -> &mut [u8] {
        match block_type(block) {
            BlockType::Common => &mut self.common_regs,
            BlockType::Socket(sn) => &mut self.socket_regs[usize::from(sn)],
            BlockType::Tx(sn) => &mut self.tx_buf[usize::from(sn)],
            BlockType::Rx(sn) => &mut self.rx_buf[usize::from(sn)],
        }
    }

    /// Returns the socket register array for a given socket.
    fn socket_regs(&mut self, socket: Socket) -> &mut [u8] {
        &mut self.socket_regs[usize::from(socket)]
    }

    /// Get the `u8` value of a socket register at the given address.
    fn socket_reg(&mut self, socket: Socket, addr: u16) -> u8 {
        self.socket_regs(socket)[usize::from(addr)]
    }

    /// Set the socket status.
    ///
    /// This is a read-only register, this method is used for internal state updates.
    fn set_sn_sr(&mut self, socket: Socket, state: SocketStatus) {
        self.socket_regs(socket)[usize::from(reg::SN_SR)] = state.into()
    }

    /// Returns the socket destination as a [`std::net`] type, without logging IO.
    fn std_sn_dest(&mut self, socket: Socket) -> std::net::SocketAddrV4 {
        let ip = std::net::Ipv4Addr::new(
            self.socket_reg(socket, reg::SN_DIPR),
            self.socket_reg(socket, reg::SN_DIPR + 1),
            self.socket_reg(socket, reg::SN_DIPR + 2),
            self.socket_reg(socket, reg::SN_DIPR + 3),
        );
        let port = u16::from_be_bytes([
            self.socket_reg(socket, reg::SN_DPORT),
            self.socket_reg(socket, reg::SN_DPORT + 1),
        ]);
        std::net::SocketAddrV4::new(ip, port)
    }

    /// Asserts that the state of the socket is `Init`
    fn socket_assert_state_init(&mut self, socket: Socket, command: SocketCommand) {
        let sn_sr = SocketStatus::try_from(self.socket_reg(socket, reg::SN_SR));
        if sn_sr != Ok(SocketStatus::Init) {
            panic!(
                "You should only send the {:?} command after initializing {:?} as TCP",
                command, socket
            )
        }
    }

    fn socket_cmd_open(&mut self, socket: Socket) -> io::Result<()> {
        // These registers are initialized by the OPEN command
        self.set_sn_rx_wr(socket, 0);
        self.nolog_set_sn_rx_rd(socket, 0);
        self.set_sn_tx_rd(socket, 0);
        self.nolog_set_sn_tx_wr(socket, 0);

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
        self.socket_assert_state_init(socket, SocketCommand::Connect);
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
                self.raise_sn_ir(socket, SocketInterrupt::DISCON_MASK);
                self.set_sn_sr(socket, SocketStatus::Closed);
            }
        }

        Ok(())
    }

    fn socket_cmd_listen(&mut self, socket: Socket) -> io::Result<()> {
        self.socket_assert_state_init(socket, SocketCommand::Listen);
        let port: u16 = self.nolog_sn_port(socket);
        let addr = std::net::SocketAddrV4::new(std::net::Ipv4Addr::LOCALHOST, port);
        log::info!("[{:?}] Opening a TCP listener on port {}", socket, addr);
        match TcpListener::bind(addr) {
            Ok(listener) => {
                log::info!("[{:?}] Bound listener on {}", socket, addr);
                listener.set_nonblocking(true)?;
                self.sockets[usize::from(socket)] = Some(SocketType::TcpListener(listener));
                self.set_sn_sr(socket, SocketStatus::Listen);
            }
            Err(e) => {
                log::warn!(
                    "[{:?}] TCP listener failed to bind to {}: {}",
                    socket,
                    addr,
                    e
                );
                self.set_sn_sr(socket, SocketStatus::Closed);
                self.raise_sn_ir(socket, SocketInterrupt::TIMEOUT_MASK);
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
            size <= self.nolog_sn_txbuf_size(socket),
            "Send data size exceeds buffer size"
        );

        let mut local_tx_buf: Vec<u8> = Vec::with_capacity(size);
        let buf = &self.tx_buf[usize::from(socket)];

        // convert the circular buffer to somthing more usable
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
                stream.flush()?;
            }
            Some(SocketType::Udp(ref mut udp)) => {
                log::info!("[{:?}] sending to {}", socket, dest);
                let num: usize = udp.send_to(&local_tx_buf, &dest)?;
                assert_eq!(num, local_tx_buf.len());
            }
            Some(SocketType::TcpListener(_)) => {
                if let Some(ref mut stream) = self.clients[usize::from(socket)] {
                    stream.write_all(&local_tx_buf)?;
                    stream.flush()?;
                }
            }
            None => {
                panic!("Unable to send data, {:?} is closed", socket)
            }
        }

        Ok(())
    }

    /// The RECV command is used to indicate that the microcontroller has read
    /// an ammount of data from the W5500, as indicated by the `sn_rx_rd`
    /// pointer.
    fn socket_cmd_recv(&mut self, _socket: Socket) -> io::Result<()> {
        // RX_RSR is automatically calculated, nothing to do here.
        Ok(())
    }

    /// `sn_port` accessor without logging IO.
    fn nolog_sn_port(&mut self, socket: Socket) -> u16 {
        u16::from_be_bytes([
            self.socket_reg(socket, reg::SN_PORT),
            self.socket_reg(socket, reg::SN_PORT + 1),
        ])
    }

    /// `sn_rx_rsr` accessor without logging IO.
    fn nolog_sn_rx_rsr(&mut self, socket: Socket) -> u16 {
        u16::from_be_bytes([
            self.socket_reg(socket, reg::SN_RX_RSR),
            self.socket_reg(socket, reg::SN_RX_RSR + 1),
        ])
    }

    /// `sn_rx_wr` accessor without logging IO.
    fn nolog_sn_rx_wr(&mut self, socket: Socket) -> u16 {
        u16::from_be_bytes([
            self.socket_reg(socket, reg::SN_RX_WR),
            self.socket_reg(socket, reg::SN_RX_WR + 1),
        ])
    }

    /// `sn_rx_rd` accessor without logging IO.
    fn nolog_sn_rx_rd(&mut self, socket: Socket) -> u16 {
        u16::from_be_bytes([
            self.socket_reg(socket, reg::SN_RX_RD),
            self.socket_reg(socket, reg::SN_RX_RD + 1),
        ])
    }

    /// `sn_tx_rd` accessor without logging IO.
    fn nolog_sn_tx_rd(&mut self, socket: Socket) -> u16 {
        u16::from_be_bytes([
            self.socket_reg(socket, reg::SN_TX_RD),
            self.socket_reg(socket, reg::SN_TX_RD + 1),
        ])
    }

    /// `sn_tx_rd` setter without logging IO.
    fn nolog_set_sn_rx_rd(&mut self, socket: Socket, ptr: u16) {
        self.socket_regs(socket)[usize::from(reg::SN_RX_RD)] = ptr.to_be_bytes()[0];
        self.socket_regs(socket)[usize::from(reg::SN_RX_RD + 1)] = ptr.to_be_bytes()[1];
    }

    /// `sn_tx_rd` setter without logging IO.
    fn nolog_set_sn_tx_wr(&mut self, socket: Socket, ptr: u16) {
        self.socket_regs(socket)[usize::from(reg::SN_TX_WR)] = ptr.to_be_bytes()[0];
        self.socket_regs(socket)[usize::from(reg::SN_TX_WR + 1)] = ptr.to_be_bytes()[1];
    }

    /// Set the socket RX write pointer register.
    ///
    /// This is a read-only register, this method is used for internal state updates.
    fn set_sn_rx_wr(&mut self, socket: Socket, wr: u16) {
        let current_wr = self.nolog_sn_rx_wr(socket);
        log::debug!(
            "[{:?}] Updating SN_RX_WR 0x{:04X} -> 0x{:04X}",
            socket,
            current_wr,
            wr
        );
        self.socket_regs(socket)[usize::from(reg::SN_RX_WR)] = wr.to_be_bytes()[0];
        self.socket_regs(socket)[usize::from(reg::SN_RX_WR + 1)] = wr.to_be_bytes()[1];
    }

    /// Set the socket TX read pointer register.
    ///
    /// This is a read-only register, this method is used for internal state updates.
    fn set_sn_tx_rd(&mut self, socket: Socket, rd: u16) {
        let current_rd = self.nolog_sn_tx_rd(socket);
        log::debug!(
            "[{:?}] Updating SN_TX_RD 0x{:04X} -> 0x{:04X}",
            socket,
            current_rd,
            rd
        );
        self.socket_regs(socket)[usize::from(reg::SN_TX_RD)] = rd.to_be_bytes()[0];
        self.socket_regs(socket)[usize::from(reg::SN_TX_RD + 1)] = rd.to_be_bytes()[1];
    }

    /// Write to the circular RX buffer given a non-circular buffer.
    fn write_rx_buf(&mut self, socket: Socket, data: &[u8]) {
        let mut rsr = self.nolog_sn_rx_rsr(socket);
        let mut wr = self.nolog_sn_rx_wr(socket);

        let buf = self.buf_from_block(socket.rx_block());
        for byte in data.iter() {
            let buf_idx = usize::from(wr) % buf.len();
            buf[buf_idx] = *byte;
            wr = wr.wrapping_add(1);
            rsr = rsr.saturating_add(1);
        }

        if rsr > u16::try_from(buf.len()).unwrap_or(u16::MAX) {
            log::warn!("[{:?}] RX buffer overflow", socket);
        }

        // rsr does not need to be set, it is calculated from wr and rd
        self.set_sn_rx_wr(socket, wr);
    }

    fn nolog_sn_rxbuf_size(&mut self, socket: Socket) -> usize {
        usize::from(self.socket_regs[usize::from(socket)][usize::from(reg::SN_RXBUF_SIZE)]) * 1024
    }

    fn nolog_sn_txbuf_size(&mut self, socket: Socket) -> usize {
        usize::from(self.socket_regs[usize::from(socket)][usize::from(reg::SN_TXBUF_SIZE)]) * 1024
    }

    fn raise_sn_ir(&mut self, socket: Socket, int: u8) {
        self.socket_regs[usize::from(socket)][usize::from(reg::SN_IR)] |= int;
    }

    fn handle_socket(&mut self, socket: Socket) -> io::Result<()> {
        let bufsize = self.nolog_sn_rxbuf_size(socket);
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
            Some(SocketType::TcpStream(ref mut stream)) => match stream.read(&mut buf) {
                Ok(num @ 1..=usize::MAX) => {
                    log::info!("[{:?}] recv {} bytes", socket, num);
                    self.write_rx_buf(socket, &buf[..num]);
                    self.raise_sn_ir(socket, SocketInterrupt::RECV_MASK);
                }
                Err(e) => match e.kind() {
                    io::ErrorKind::WouldBlock => {}
                    _ => return Err(e),
                },
                _ => {}
            },
            Some(SocketType::TcpListener(ref mut listener)) => {
                if let Some(ref mut stream) = self.clients[usize::from(socket)] {
                    match stream.read(&mut buf) {
                        Ok(num @ 1..=usize::MAX) => {
                            log::info!("[{:?}] recv {} bytes", socket, num);
                            self.write_rx_buf(socket, &buf[..num]);
                            self.raise_sn_ir(socket, SocketInterrupt::RECV_MASK);
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
                            log::info!("[{:?}] Accepted a new stream from {}", socket, addr);
                            self.raise_sn_ir(socket, SocketInterrupt::CON_MASK);
                            self.set_sn_sr(socket, SocketStatus::Established);
                            stream.set_nonblocking(true)?;
                            self.clients[usize::from(socket)] = Some(stream);
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
            SocketCommand::Listen => self.socket_cmd_listen(socket)?,
            _ => unimplemented!("Socket command {:?} sent to {:?}", cmd, socket),
        }
        Ok(())
    }

    fn set_sn_tx_wr(&mut self, socket: Socket, ptr: u16) -> Result<(), Self::Error> {
        // sn_tx_fsr is automatically calculated as the difference between
        // sn_tx_wr and sn_tx_rd
        let mut sr = self.socket_regs[usize::from(socket)];
        let mut fsr = self.sn_tx_fsr(socket).unwrap();
        fsr = fsr.saturating_sub(ptr);
        let fsr_bytes = u16::to_be_bytes(fsr);
        sr[usize::from(reg::SN_TX_FSR)] = fsr_bytes[0];
        sr[usize::from(reg::SN_TX_FSR) + 1] = fsr_bytes[1];

        self.write(reg::SN_TX_WR, socket.block(), &ptr.to_be_bytes())?;
        Ok(())
    }

    fn sn_ir(&mut self, socket: Socket) -> Result<SocketInterrupt, Self::Error> {
        self.handle_socket(socket)?;
        let mut reg: [u8; 1] = [0];
        self.read(reg::SN_IR, socket.block(), &mut reg)?;
        Ok(SocketInterrupt::from(reg[0]))
    }

    fn set_sn_ir(&mut self, socket: Socket, ir: SocketInterrupt) -> Result<(), Self::Error> {
        let sn_ir_reg: &mut u8 =
            &mut self.socket_regs[usize::from(socket)][usize::from(reg::SN_IR)];

        let log_cleared = |name: &str| {
            log::trace!(
                "[W] [SN{}] [0x{:02X}] ({}) clearing {}",
                u8::from(socket),
                reg::SN_IR,
                regmap::socket_reg_name(&reg::SN_IR),
                name
            )
        };

        if ir.con_raised() {
            log_cleared("CON");
            *sn_ir_reg &= !SocketInterrupt::CON_MASK;
        }
        if ir.discon_raised() {
            log_cleared("DISCON");
            *sn_ir_reg &= !SocketInterrupt::DISCON_MASK;
        }
        if ir.recv_raised() {
            log_cleared("RECV");
            *sn_ir_reg &= !SocketInterrupt::RECV_MASK;
        }
        if ir.timeout_raised() {
            log_cleared("TIMEOUT");
            *sn_ir_reg &= !SocketInterrupt::TIMEOUT_MASK;
        }
        if ir.sendok_raised() {
            log_cleared("SENDOK");
            *sn_ir_reg &= !SocketInterrupt::SENDOK_MASK;
        }

        Ok(())
    }

    fn sn_rx_rsr(&mut self, socket: Socket) -> Result<u16, Self::Error> {
        self.handle_socket(socket)?;

        // sn_rx_rsr is automatically calculated as the difference between sn_rx_rd and sn_rx_wr
        // this does the updating before reading
        let rsr: u16 = {
            let rx_rd: u16 = self.nolog_sn_rx_rd(socket);
            let rx_wr: u16 = self.nolog_sn_rx_wr(socket);

            if rx_wr >= rx_rd {
                rx_wr - rx_rd
            } else {
                u16::try_from(self.nolog_sn_rxbuf_size(socket)).unwrap() - rx_wr + rx_rd
            }
        };

        self.socket_regs(socket)[usize::from(reg::SN_RX_RSR)] = rsr.to_be_bytes()[0];
        self.socket_regs(socket)[usize::from(reg::SN_RX_RSR + 1)] = rsr.to_be_bytes()[1];

        // a `read` call is still used so that this gets logged
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
