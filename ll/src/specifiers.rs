//! Register specifiers (enumerations).

use core::convert::TryFrom;

/// Errors that occur upon converting `u8` to an enumeration.
#[derive(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Hash)]
pub struct ValueError {
    /// Errenous value.
    pub val: u8,
}

impl ValueError {
    /// Create a new `ValueError`.
    pub const fn new(val: u8) -> ValueError {
        ValueError { val }
    }
}

/// Socket status.
#[derive(Copy, Clone, Eq, PartialEq, Debug, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum SocketStatus {
    /// This indicates that Socket n is released.
    /// When DISCON, CLOSE command is ordered, or when a timeout occurs,
    /// it is changed to SOCK_CLOSED regardless of previous status.
    Closed = 0x00,
    /// This indicates Socket n is opened with TCP mode.
    /// It is changed to SOCK_INIT when Sn_MR (P\[3:0\]) = `0b0001` and
    /// OPEN command is ordered.After SOCK_INIT,
    /// user can use LISTEN / CONNECT command.
    Init = 0x13,
    /// This indicates Socket n is operating as "TCP server" mode and
    /// waiting for connection-request (SYN  packet) from a peer
    /// ("TCP client").
    /// It will change to SOCK_ESTALBLISHED when the connection-request is
    /// successfully accepted.
    /// Otherwise it will change to SOCK_CLOSED after TCP<sub>TO</sub>
    /// occurred (Sn_IR(TIMEOUT) = `1`).
    Listen = 0x14,
    /// Temporary status between status transitions.
    SynSent = 0x15,
    /// Temporary status between status transitions.
    SynRecv = 0x16,
    /// This indicates the status of the connection of Socket n.
    /// It changes to SOCK_ESTABLISHED when the "TCP SERVER" processed the
    /// SYN packet from the "TCP CLIENT" during SOCK_LISTEN, or when the
    /// CONNECT command is successful.
    /// During SOCK_ESTABLISHED, DATA packet can be transferred using
    /// SEND or RECV command.
    Established = 0x17,
    /// Temporary status between status transitions.
    FinWait = 0x18,
    /// Temporary status between status transitions.
    Closing = 0x1A,
    /// Temporary status between status transitions.
    TimeWait = 0x1B,
    /// This indicates Socket n received the disconnect-request (FIN packet)
    /// from the connected peer.
    ///
    /// This is half-closing status, and data can be transferred.
    ///
    /// For full-closing the DISCON command is used.
    ///
    /// For just-closing the CLOSE command is used.
    CloseWait = 0x1C,
    /// Temporary status between status transitions.
    LastAck = 0x1D,
    /// This indicates Socket n is opened in UDP mode
    /// (Sn_MR(P\[3:0\]) = `0010`).
    /// It changes to SOCK_UDP when Sn_MR(P\[3:0\]) = `0010` and
    /// the OPEN command is ordered.
    /// Unlike TCP mode, data can be transfered without the
    /// connection-process.
    Udp = 0x22,
    /// This indicates socket 0 is opened in MACRAW mode and is valid only
    /// in socket 0.
    /// It changes to SOCK_MACRAW when S0_MR(P\[3:0\] = `0100`) and the
    /// OPEN command is ordered.
    ///
    /// The MACRAW mode can transfer a MAC packet (Ethernet frame) without
    /// the connection-process.
    Macraw = 0x42,
}
impl From<SocketStatus> for u8 {
    fn from(val: SocketStatus) -> u8 {
        val as u8
    }
}
impl TryFrom<u8> for SocketStatus {
    type Error = ValueError;
    fn try_from(val: u8) -> Result<SocketStatus, ValueError> {
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
            _ => Err(ValueError::new(val)),
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
    /// Socket n is initialized and opened according to the protocol
    /// selected in [Sn_MR].
    ///
    /// | [Sn_MR]        | [Sn_SR]       |
    /// |----------------|---------------|
    /// | [Sn_MR_CLOSE]  | -             |
    /// | [Sn_MR_TCP]    | [SOCK_INIT]   |
    /// | [Sn_MR_UDP]    | [SOCK_UDP]    |
    /// | [S0_MR_MACRAW] | [SOCK_MACRAW] |
    ///
    /// [Sn_MR]: crate::Registers::sn_mr
    /// [Sn_Sr]: crate::Registers::sn_sr
    /// [Sn_MR_CLOSE]: enum.Protocol.html#variant.Closed
    /// [Sn_MR_TCP]: enum.Protocol.html#variant.Tcp
    /// [Sn_MR_UDP]: enum.Protocol.html#variant.Udp
    /// [S0_MR_MACRAW]: enum.Protocol.html#variant.Macraw
    /// [SOCK_INIT]: enum.SocketStatus.html#variant.Init
    /// [SOCK_UDP]: enum.SocketStatus.html#variant.Udp
    /// [SOCK_MACRAW]: enum.SocketStatus.html#variant.Macraw
    Open = 0x01,
    /// Socket n operates as a "TCP server" and waits for connection-request
    /// (SYN packet) from any "TCP client".
    /// The Sn_SR changes the state from SOCK_INIT to SOCKET_LISTEN.
    ///
    /// When a "TCP client" connection request is successfully established,
    /// the Sn_SR changes from SOCK_LISTEN to SOCK_ESTABLISHED and the
    /// Sn_IR(0) becomes `1`.
    /// But when a "TCP client" connection request is failed,
    /// Sn_IR(3) becomes `1` and the status of Sn_SR changes to SOCK_CLOSED.
    ///
    /// Only valid in [TCP](enum.Protocol.html#variant.Tcp) mode.
    Listen = 0x02,
    /// To connect, a connect-request (SYN packet) is sent to "TCP server"
    /// configured by Sn_DIPR & Sn_DPORT (destination address & port).
    /// If the connect-request is successful, the Sn_SR is changed to
    /// SOCK_ESTABLISHED and the Sn_IR(0) becomes `1`.
    ///
    /// The connect-request fails in the following three cases:
    /// 1. When a ARP<sub>TO</sub> occurs (Sn_IR(3)=`1`) because the
    ///    destination hardware address is not acquired through the
    ///    ARP-process.
    /// 2. When a SYN/ACK packet is not received and TCP<sub>TO</sub>
    ///   (Sn_IR(3)= `1`)
    /// 3. When a RST packet is received instead of a SYN/ACK packet.
    ///
    /// In these cases, Sn_SR is changed to SOCK_CLOSED.
    ///
    /// Only valid in [TCP](enum.Protocol.html#variant.Tcp) mode when acting
    /// as a "TCP client".
    Connect = 0x04,
    /// Regardless  of "TCP  server" or "TCP  client", the DISCON command
    /// processes the disconnect-process
    /// ("Active close" or "Passive close").
    ///
    /// * **Active close** it transmits disconnect-request(FIN packet)
    ///   to theconnected peer.
    /// * **Passive close** when FIN packet is received from peer,
    ///   a FIN packet is replied back to the peer.
    ///
    /// When the disconnect-process is successful
    /// (that is, FIN/ACK packet is received successfully),
    /// Sn_SR is changed to SOCK_CLOSED.
    /// Otherwise, TCP<sub>TO</sub>occurs (Sn_IR(3)=`1`) and then Sn_SR is
    /// changed to SOCK_CLOSED.
    ///
    /// If CLOSE is used instead of DISCON, only Sn_SR is changed to
    /// SOCK_CLOSED without disconnect-process.
    /// If a RST packet is received from a peer during communication, Sn_SR
    /// is unconditionally changed to SOCK_CLOSED.
    ///
    /// Only valid in [TCP](enum.Protocol.html#variant.Tcp) mode.
    Disconnect = 0x08,
    /// Close socket n.
    ///
    /// Sn_SR is changed to SOCK_CLOSED.
    Close = 0x10,
    /// Transmits all the data in the Socket n TX buffer.
    Send = 0x20,
    /// The basic operation is same as SEND.
    /// Normally SEND transmits data after destination hardware address is
    /// acquired by the automatic ARP-process (Address Resolution Protocol).
    /// But SEND_MAC transmits data without the automatic ARP-process.
    /// In this case, the destination hardware address is acquired from
    /// Sn_DHAR configured by host, instead of APR-process.
    ///
    /// Only valid in [UDP](enum.Protocol.html#variant.Udp) mode.
    SendMac = 0x21,
    /// Checks the connection status by sending a 1 byte keep-alive packet.
    /// If the peer cannot respond to the keep-alive packet during timeout
    /// time, the connection is terminated and the timeout interrupt will
    /// occur.
    ///
    /// Only valid in [TCP](enum.Protocol.html#variant.Tcp) mode.
    SendKeep = 0x22,
    /// RECV completes the processing of the received data in Socket n RX
    /// Buffer by using a RX read pointer register (Sn_RX_RD).
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
    type Error = ValueError;
    fn try_from(val: u8) -> Result<Protocol, ValueError> {
        match val {
            x if x == Protocol::Closed as u8 => Ok(Protocol::Closed),
            x if x == Protocol::Tcp as u8 => Ok(Protocol::Tcp),
            x if x == Protocol::Udp as u8 => Ok(Protocol::Udp),
            x if x == Protocol::Macraw as u8 => Ok(Protocol::Macraw),
            _ => Err(ValueError::new(val)),
        }
    }
}

/// PHY operation mode.
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
    type Error = ValueError;
    fn try_from(val: u8) -> Result<OperationMode, ValueError> {
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
            _ => Err(ValueError::new(val)),
        }
    }
}
impl Default for OperationMode {
    fn default() -> OperationMode {
        OperationMode::Auto
    }
}

/// PHY link status.
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
    type Error = ValueError;
    fn try_from(val: u8) -> Result<BufferSize, ValueError> {
        match val {
            x if x == BufferSize::KB0 as u8 => Ok(BufferSize::KB0),
            x if x == BufferSize::KB1 as u8 => Ok(BufferSize::KB1),
            x if x == BufferSize::KB2 as u8 => Ok(BufferSize::KB2),
            x if x == BufferSize::KB4 as u8 => Ok(BufferSize::KB0),
            x if x == BufferSize::KB8 as u8 => Ok(BufferSize::KB0),
            x if x == BufferSize::KB16 as u8 => Ok(BufferSize::KB0),
            _ => Err(ValueError::new(val)),
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
    /// assert_eq!(BufferSize::KB2.size_in_bytes(), 2048);
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
