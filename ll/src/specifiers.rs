//! Register specifiers (enumerations).

use core::convert::TryFrom;

/// Socket status.
///
/// This is used with [`crate::Registers::sn_sr`].
#[derive(Copy, Clone, Eq, PartialEq, Debug, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum SocketStatus {
    /// Socket closed, this is the reset state of all sockets.
    ///
    /// This state can be set by a [`SocketCommand::Disconnect`] or
    /// [`SocketCommand::Close`] command.
    ///
    /// This state will also be set automatically if a timeout occurs.
    Closed = 0x00,
    /// The socket is opened in TCP mode.
    ///
    /// This state is set when the socket protocol is [`Protocol::Tcp`], and a
    /// [`SocketCommand::Open`] command is sent.
    ///
    /// In this state you can use the [`SocketCommand::Listen`] and
    /// [`SocketCommand::Connect`] commands.
    Init = 0x13,
    /// The socket is listening, operating as a TCP server.
    ///
    /// The socket will wait for a connextion-request (SYN packet) from a
    /// peer (TCP client).
    ///
    /// The state will change to [`SocketStatus::Established`] when the
    /// connection-request is successfully accepted.
    /// Otherwise the state will change to [`SocketStatus::Closed`] after the
    /// TCP timeout duration set by
    /// [`crate::Registers::rcr`] and [`crate::Registers::rtr`].
    Listen = 0x14,
    /// Connection request (SYN packet) has been sent to a peer.
    ///
    /// This is temporarily displayed between the [`SocketStatus::Init`] and
    /// [`SocketStatus::Established`] states, after a [`SocketCommand::Connect`]
    /// command has been sent.
    ///
    /// If the SYN/ACK is received from the peer the state changes to
    /// [`SocketStatus::Established`], otherwise the state changes to
    /// [`SocketStatus::Closed`] after the TCP timeout duration set by
    /// [`crate::Registers::rcr`] and [`crate::Registers::rtr`].
    SynSent = 0x15,
    /// Connection request (SYN packet) has been received from a peer.
    ///
    /// If the socket sends the response (SYN/ACK packet) to the peer
    /// successfully the state changes to [`SocketStatus::Established`],
    /// otherwise the state changes to [`SocketStatus::Closed`] after the TCP
    /// timeout duration set by [`crate::Registers::rcr`] and
    /// [`crate::Registers::rtr`].
    SynRecv = 0x16,
    /// TCP connection is established.
    ///
    /// When operating as a TCP client this state is set after the TCP server
    /// accepts the SYN packet, which is sent by the client after issuing a
    /// [`SocketCommand::Connect`].
    ///
    /// When operating as a TCP server this state is set after a client
    /// connects when in the [`SocketStatus::Listen`] state.
    ///
    /// While in this state data can be transfered with the
    /// [`SocketCommand::Send`] and [`SocketCommand::Recv`] commands.
    Established = 0x17,
    /// Temporary status between status transitions.
    ///
    /// This indicates the socket is closing.
    FinWait = 0x18,
    /// Temporary status between status transitions.
    ///
    /// This indicates the socket is closing.
    Closing = 0x1A,
    /// Temporary status between status transitions.
    ///
    /// This indicates the socket is closing.
    TimeWait = 0x1B,
    /// The socket has received the disconnect-request (FIN pakcet) from the
    /// connected peer.
    ///
    /// This is half-closing status, and data can be transferred.
    ///
    /// For full-closing the [`SocketCommand::Disconnect`] command is used.
    ///
    /// For just-closing the [`SocketCommand::Close`] command is used.
    CloseWait = 0x1C,
    /// Temporary status between status transitions.
    LastAck = 0x1D,
    /// Socket is opened in UDP mode.
    ///
    /// This state is set when the socket protocol is [`Protocol::Udp`], and a
    /// [`SocketCommand::Open`] command is sent.
    Udp = 0x22,
    /// Socket is opened in MACRAW mode.
    ///
    /// This is valid only for socket 0.
    ///
    /// This state is set when the socket protocol is [`Protocol::Macraw`], and
    /// a [`SocketCommand::Open`] command is sent.
    Macraw = 0x42,
}
impl From<SocketStatus> for u8 {
    fn from(val: SocketStatus) -> u8 {
        val as u8
    }
}
impl TryFrom<u8> for SocketStatus {
    type Error = u8;
    fn try_from(val: u8) -> Result<SocketStatus, u8> {
        match val {
            x if x == SocketStatus::Closed as u8 => Ok(SocketStatus::Closed),
            x if x == SocketStatus::Init as u8 => Ok(SocketStatus::Init),
            x if x == SocketStatus::Listen as u8 => Ok(SocketStatus::Listen),
            x if x == SocketStatus::SynSent as u8 => Ok(SocketStatus::SynSent),
            x if x == SocketStatus::SynRecv as u8 => Ok(SocketStatus::SynRecv),
            x if x == SocketStatus::Established as u8 => Ok(SocketStatus::Established),
            x if x == SocketStatus::FinWait as u8 => Ok(SocketStatus::FinWait),
            x if x == SocketStatus::Closing as u8 => Ok(SocketStatus::Closing),
            x if x == SocketStatus::TimeWait as u8 => Ok(SocketStatus::TimeWait),
            x if x == SocketStatus::CloseWait as u8 => Ok(SocketStatus::CloseWait),
            x if x == SocketStatus::LastAck as u8 => Ok(SocketStatus::LastAck),
            x if x == SocketStatus::Udp as u8 => Ok(SocketStatus::Udp),
            x if x == SocketStatus::Macraw as u8 => Ok(SocketStatus::Macraw),
            _ => Err(val),
        }
    }
}

impl Default for SocketStatus {
    fn default() -> Self {
        SocketStatus::Closed
    }
}

/// Socket commands.
///
/// This is used to set the command for socket n.
///
/// After W5500 accepts the command, the [`crate::Registers::set_sn_cr`]
/// register is automatically cleared to 0x00.
/// Even though [`crate::Registers::set_sn_cr`] is cleared to 0x00, the command
/// is still being processed.
/// To check whether the command is completed or not, check
/// [`crate::Registers::sn_ir`] or [`crate::Registers::sn_sr`].
#[derive(Copy, Clone, Eq, PartialEq, Debug, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum SocketCommand {
    /// The command register clears to this state once a command has been
    /// accepted.
    Accepted = 0x00,
    /// The socket is initialized and opened according to the protocol
    /// selected in [`crate::Registers::sn_mr`].
    ///
    /// | [`crate::Registers::sn_mr`] | [`crate::Registers::sn_sr`] |
    /// |-----------------------------|-----------------------------|
    /// | [`Protocol::Closed`]        | -                           |
    /// | [`Protocol::Tcp`]           | [`SocketStatus::Init`]      |
    /// | [`Protocol::Udp`]           | [`SocketStatus::Udp`]       |
    /// | [`Protocol::Macraw`]        | [`SocketStatus::Macraw`]    |
    Open = 0x01,
    /// Operate the socket as a TCP server.
    ///
    /// This will change the socket state from [`SocketStatus::Init`] to
    /// [`SocketStatus::Listen`], and the socket will listen for a
    /// connection-request (SYN packet) from any TCP client.
    ///
    /// When a TCP client connection request is successfully established,
    /// the socket state changes from [`SocketStatus::Listen`] to
    /// [`SocketStatus::Established`] and the `CON` socket interrupt is raised
    /// ([`crate::SocketInterrupt::con_raised`]).
    ///
    /// When a TCP client connection request fails the `TIMEOUT` socket
    /// interrupt is set ([`crate::SocketInterrupt::timeout_raised`]) and the
    /// socket status changes to [`SocketStatus::Closed`].
    ///
    /// Only valid in [`Protocol::Tcp`] mode.
    Listen = 0x02,
    /// Connect to a TCP server.
    ///
    /// A connect-request (SYN packet) is sent to the TCP server configured by
    /// [`crate::Registers::sn_dipr`] and [`crate::Registers::sn_dport`]
    /// (destination IPv4 address and port).
    ///
    /// If the connect-request is successful, the socket state changes to
    /// [`SocketStatus::Established`] and the `CON` socket interrupt is raised
    /// ([`crate::SocketInterrupt::con_raised`]).
    ///
    /// The connect-request fails in the following three cases:
    /// 1. When a ARP<sub>TO</sub> occurs
    ///    ([`crate::SocketInterrupt::con_raised`]) because the
    ///    destination hardware address is not acquired through the
    ///    ARP-process.
    /// 2. When a SYN/ACK packet is not received within the TCP timeout duration
    ///    set by [`crate::Registers::rcr`] and [`crate::Registers::rtr`]
    ///    ([`crate::SocketInterrupt::timeout_raised`]).
    /// 3. When a RST packet is received instead of a SYN/ACK packet.
    ///
    /// In these cases the socket state changes to [`SocketStatus::Closed`].
    ///
    /// Only valid in [`Protocol::Tcp`] mode when acting as a TCP client.
    Connect = 0x04,
    /// Start the disconnect process.
    ///
    /// * **Active close** it transmits disconnect-request(FIN packet)
    ///   to the connected peer.
    /// * **Passive close** when FIN packet is received from peer,
    ///   a FIN packet is replied back to the peer.
    ///
    /// When the disconnect-process is successful
    /// (that is, FIN/ACK packet is received successfully),
    /// the socket state changes to [`SocketStatus::Closed`].
    /// Otherwise, TCP timeout occurs
    /// ([`crate::SocketInterrupt::timeout_raised`]) and then
    /// the socket state changes to [`SocketStatus::Closed`].
    ///
    /// If [`SocketCommand::Close`] is used instead of
    /// [`SocketCommand::Disconnect`], the socket state is changes to
    /// [`SocketStatus::Closed`] without the disconnect process.
    ///
    /// If a RST packet is received from a peer during communication the socket
    /// status is unconditionally changed to [`SocketStatus::Closed`].
    ///
    /// Only valid in [`Protocol::Tcp`] mode.
    Disconnect = 0x08,
    /// Close the socket.
    ///
    /// The socket status is changed to [`SocketStatus::Closed`].
    Close = 0x10,
    /// Transmits all the data in the socket TX buffer.
    Send = 0x20,
    /// The basic operation is same as [`SocketCommand::Send`].
    ///
    /// Normally [`SocketCommand::Send`] transmits data after destination
    /// hardware address is acquired by the automatic ARP-process
    /// (Address Resolution Protocol).
    /// [`SocketCommand::SendMac`] transmits data without the automatic
    /// ARP-process.
    /// In this case, the destination hardware address is acquired from
    /// [`crate::Registers::sn_dhar`] configured by the host, instead of the ARP
    /// process.
    ///
    /// Only valid in [`Protocol::Udp`] mode.
    SendMac = 0x21,
    /// Sends a 1 byte keep-alive packet.
    ///
    /// If the peer cannot respond to the keep-alive packet during timeout
    /// time, the connection is terminated and the timeout interrupt will
    /// occur ([`crate::SocketInterrupt::timeout_raised`]).
    ///
    /// Only valid in [`Protocol::Tcp`] mode.
    SendKeep = 0x22,
    /// RECV completes the processing of the received data in socket RX
    /// buffer.
    ///
    /// See [`crate::Registers::sn_rx_buf`] for an example.
    Recv = 0x40,
}
impl From<SocketCommand> for u8 {
    fn from(val: SocketCommand) -> u8 {
        val as u8
    }
}

/// Socket protocol.
///
/// Used in the [`crate::SocketMode`] register.
#[derive(Copy, Clone, Eq, PartialEq, Debug, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Protocol {
    /// Closed.
    Closed = 0b0000,
    /// TCP.
    Tcp = 0b0001,
    /// UDP.
    Udp = 0b0010,
    /// MACRAW.
    ///
    /// MACRAW mode can only be used with socket 0.
    Macraw = 0b0100,
}
impl From<Protocol> for u8 {
    fn from(val: Protocol) -> u8 {
        val as u8
    }
}
impl Default for Protocol {
    fn default() -> Protocol {
        Protocol::Closed
    }
}
impl TryFrom<u8> for Protocol {
    type Error = u8;
    fn try_from(val: u8) -> Result<Protocol, u8> {
        match val {
            x if x == Protocol::Closed as u8 => Ok(Protocol::Closed),
            x if x == Protocol::Tcp as u8 => Ok(Protocol::Tcp),
            x if x == Protocol::Udp as u8 => Ok(Protocol::Udp),
            x if x == Protocol::Macraw as u8 => Ok(Protocol::Macraw),
            _ => Err(val),
        }
    }
}

/// PHY operation mode.
///
/// This is used by [`crate::PhyCfg`] for the
/// [`crate::Registers::set_phycfgr`] and [`crate::Registers::phycfgr`] methods.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Hash)]
#[repr(u8)]
pub enum OperationMode {
    /// 10BT half-duplex. Auto-negotiation disabled.
    HalfDuplex10bt = 0b000,
    /// 10BT full-duplex. Auto-negotiation disabled.
    FullDuplex10bt = 0b001,
    /// 100BT half-duplex. Auto-negotiation disabled.
    HalfDuplex100bt = 0b010,
    /// 100BT full-duplex. Auto-negotiation disabled.
    FullDuplex100bt = 0b011,
    /// 100BT half-duplex. Auto-negotiation enabled.
    HalfDuplex100btAuto = 0b100,
    /// Power down mode.
    PowerDown = 0b110,
    /// All capable. Auto-negotiation enabled.
    Auto = 0b111,
}
impl From<OperationMode> for u8 {
    fn from(val: OperationMode) -> u8 {
        val as u8
    }
}
impl TryFrom<u8> for OperationMode {
    type Error = u8;
    fn try_from(val: u8) -> Result<OperationMode, u8> {
        match val {
            x if x == OperationMode::HalfDuplex10bt as u8 => Ok(OperationMode::HalfDuplex10bt),
            x if x == OperationMode::FullDuplex10bt as u8 => Ok(OperationMode::FullDuplex10bt),
            x if x == OperationMode::HalfDuplex100bt as u8 => Ok(OperationMode::HalfDuplex100bt),
            x if x == OperationMode::FullDuplex100bt as u8 => Ok(OperationMode::FullDuplex100bt),
            x if x == OperationMode::HalfDuplex100btAuto as u8 => {
                Ok(OperationMode::HalfDuplex100btAuto)
            }
            x if x == OperationMode::PowerDown as u8 => Ok(OperationMode::PowerDown),
            x if x == OperationMode::Auto as u8 => Ok(OperationMode::Auto),
            _ => Err(val),
        }
    }
}
impl Default for OperationMode {
    fn default() -> OperationMode {
        OperationMode::Auto
    }
}

/// PHY link status.
///
/// This is used by [`crate::PhyCfg`] for the
/// [`crate::Registers::set_phycfgr`] and [`crate::Registers::phycfgr`] methods.
#[derive(Copy, Clone, Eq, PartialEq, Debug, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum LinkStatus {
    /// PHY link down.
    Down = 0,
    /// PHY link up.
    Up = 1,
}
impl From<bool> for LinkStatus {
    fn from(val: bool) -> LinkStatus {
        if val {
            LinkStatus::Up
        } else {
            LinkStatus::Down
        }
    }
}
impl From<LinkStatus> for u8 {
    fn from(val: LinkStatus) -> u8 {
        val as u8
    }
}

/// PHY speed status.
///
/// This is used by [`crate::PhyCfg`] for the
/// [`crate::Registers::set_phycfgr`] and [`crate::Registers::phycfgr`] methods.
#[derive(Copy, Clone, Eq, PartialEq, Debug, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum SpeedStatus {
    /// 10 Mbps.
    Mbps10 = 0,
    /// 100 Mbps.
    Mbps100 = 1,
}
impl From<bool> for SpeedStatus {
    fn from(val: bool) -> SpeedStatus {
        if val {
            SpeedStatus::Mbps100
        } else {
            SpeedStatus::Mbps10
        }
    }
}
impl From<SpeedStatus> for u8 {
    fn from(val: SpeedStatus) -> u8 {
        val as u8
    }
}

/// PHY duplex status.
///
/// This is used by [`crate::PhyCfg`] for the
/// [`crate::Registers::set_phycfgr`] and [`crate::Registers::phycfgr`] methods.
#[derive(Copy, Clone, Eq, PartialEq, Debug, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum DuplexStatus {
    /// Half duplex
    Half = 0,
    /// Full duplex.
    Full = 1,
}
impl From<bool> for DuplexStatus {
    fn from(val: bool) -> DuplexStatus {
        if val {
            DuplexStatus::Full
        } else {
            DuplexStatus::Half
        }
    }
}
impl From<DuplexStatus> for u8 {
    fn from(val: DuplexStatus) -> u8 {
        val as u8
    }
}

/// RX and TX buffer sizes.
///
/// This is an argument of [`crate::Registers::set_sn_rxbuf_size`] and
/// [`crate::Registers::set_sn_txbuf_size`]
#[derive(Copy, Clone, Eq, PartialEq, PartialOrd, Ord, Hash, Debug)]
#[repr(u8)]
pub enum BufferSize {
    /// 0 KiB
    KB0 = 0,
    /// 1 KiB
    KB1 = 1,
    /// 2 KiB
    KB2 = 2,
    /// 4 KiB
    KB4 = 4,
    /// 8 KiB
    KB8 = 8,
    /// 16 KiB
    KB16 = 16,
}
impl From<BufferSize> for u8 {
    fn from(val: BufferSize) -> u8 {
        val as u8
    }
}
impl TryFrom<u8> for BufferSize {
    type Error = u8;
    fn try_from(val: u8) -> Result<BufferSize, u8> {
        match val {
            x if x == BufferSize::KB0 as u8 => Ok(BufferSize::KB0),
            x if x == BufferSize::KB1 as u8 => Ok(BufferSize::KB1),
            x if x == BufferSize::KB2 as u8 => Ok(BufferSize::KB2),
            x if x == BufferSize::KB4 as u8 => Ok(BufferSize::KB0),
            x if x == BufferSize::KB8 as u8 => Ok(BufferSize::KB0),
            x if x == BufferSize::KB16 as u8 => Ok(BufferSize::KB0),
            _ => Err(val),
        }
    }
}

impl Default for BufferSize {
    fn default() -> Self {
        BufferSize::KB2
    }
}

impl BufferSize {
    /// Get the buffer size in bytes.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::BufferSize;
    ///
    /// assert_eq!(BufferSize::KB0.size_in_bytes(), 0);
    /// assert_eq!(BufferSize::KB1.size_in_bytes(), 1 * 1024);
    /// assert_eq!(BufferSize::KB2.size_in_bytes(), 2 * 1024);
    /// assert_eq!(BufferSize::KB4.size_in_bytes(), 4 * 1024);
    /// assert_eq!(BufferSize::KB8.size_in_bytes(), 8 * 1024);
    /// assert_eq!(BufferSize::KB16.size_in_bytes(), 16 * 1024);
    /// ```
    pub const fn size_in_bytes(&self) -> usize {
        match self {
            BufferSize::KB0 => 0,
            BufferSize::KB1 => 1024,
            BufferSize::KB2 => 2048,
            BufferSize::KB4 => 4096,
            BufferSize::KB8 => 8192,
            BufferSize::KB16 => 16384,
        }
    }
}
