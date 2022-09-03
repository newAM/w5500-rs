//! Register specifiers (enumerations).

/// Socket status.
///
/// This is used with the [`sn_sr`] method.
///
/// [`sn_sr`]: crate::Registers::sn_sr
#[derive(Copy, Clone, Eq, PartialEq, Debug, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "ufmt", derive(ufmt::derive::uDebug))]
#[repr(u8)]
pub enum SocketStatus {
    /// Socket closed, this is the reset state of all sockets.
    ///
    /// This state can be set by a [`Disconnect`] or [`Close`] command.
    ///
    /// This state will also be set automatically if a timeout occurs.
    ///
    /// [`Disconnect`]: SocketCommand::Disconnect
    /// [`Close`]: SocketCommand::Close
    Closed = 0x00,
    /// The socket is opened in TCP mode.
    ///
    /// This state is set when the socket protocol is [`Tcp`], and a [`Open`]
    /// command is sent.
    ///
    /// In this state you can use the [`Listen`] and [`Connect`] commands.
    ///
    /// [`Tcp`]: Protocol::Tcp
    /// [`Open`]: SocketCommand::Open
    /// [`Listen`]: SocketCommand::Listen
    /// [`Connect`]: SocketCommand::Connect
    Init = 0x13,
    /// The socket is listening, operating as a TCP server.
    ///
    /// The socket will wait for a connextion-request (SYN packet) from a
    /// peer (TCP client).
    ///
    /// The state will change to [`Established`] when the connection-request is
    /// successfully accepted.
    /// Otherwise the state will change to [`Closed`] after the
    /// TCP timeout duration set by [`rcr`] and [`rtr`].
    ///
    /// [`Established`]: SocketStatus::Established
    /// [`Closed`]: SocketStatus::Closed
    /// [`rcr`]: crate::Registers::rcr
    /// [`rtr`]: crate::Registers::rtr
    Listen = 0x14,
    /// Connection request (SYN packet) has been sent to a peer.
    ///
    /// This is temporarily displayed between the [`Init`] and [`Established`]
    /// states, after a [`Connect`] command has been sent.
    ///
    /// If the SYN/ACK is received from the peer the state changes to
    /// [`Established`], otherwise the state changes to [`Closed`] after the TCP
    /// timeout duration set by [`rcr`] and [`rtr`].
    ///
    /// [`Init`]: SocketStatus::Init
    /// [`Connect`]: SocketCommand::Connect
    /// [`Established`]: SocketStatus::Established
    /// [`Closed`]: SocketStatus::Closed
    /// [`rcr`]: crate::Registers::rcr
    /// [`rtr`]: crate::Registers::rtr
    SynSent = 0x15,
    /// Connection request (SYN packet) has been received from a peer.
    ///
    /// If the socket sends the response (SYN/ACK packet) to the peer
    /// successfully the state changes to [`Established`], otherwise the state
    /// changes to [`Closed`] after the TCP timeout duration set by [`rcr`] and
    /// [`rtr`].
    ///
    /// [`Established`]: SocketStatus::Established
    /// [`Closed`]: SocketStatus::Closed
    /// [`rcr`]: crate::Registers::rcr
    /// [`rtr`]: crate::Registers::rtr
    SynRecv = 0x16,
    /// TCP connection is established.
    ///
    /// When operating as a TCP client this state is set after the TCP server
    /// accepts the SYN packet, which is sent by the client after issuing a
    /// [`Connect`].
    ///
    /// When operating as a TCP server this state is set after a client
    /// connects when in the [`Listen`] state.
    ///
    /// While in this state data can be transferred with the [`Send`] and
    /// [`Recv`] commands.
    ///
    /// [`Connect`]: SocketCommand::Connect
    /// [`Listen`]: SocketStatus::Listen
    /// [`Send`]: SocketCommand::Send
    /// [`Recv`]: SocketCommand::Recv
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
    /// The socket has received the disconnect-request (FIN packet) from the
    /// connected peer.
    ///
    /// This is half-closing status, and data can be transferred.
    ///
    /// For full-closing the [`Disconnect`] command is used.
    ///
    /// For just-closing the [`Close`] command is used.
    ///
    /// [`Disconnect`]: SocketCommand::Disconnect
    /// [`Close`]: SocketCommand::Close
    CloseWait = 0x1C,
    /// Temporary status between status transitions.
    LastAck = 0x1D,
    /// Socket is opened in UDP mode.
    ///
    /// This state is set when the socket protocol is [`Udp`], and a [`Open`]
    /// command is sent.
    ///
    /// [`Udp`]: Protocol::Udp
    /// [`Open`]: SocketCommand::Open
    Udp = 0x22,
    /// Socket is opened in MACRAW mode.
    ///
    /// This is valid only for [socket 0].
    ///
    /// This state is set when the socket protocol is [`Macraw`], and a [`Open`]
    /// command is sent.
    ///
    /// [socket 0]: crate::Sn::Sn0
    /// [`Macraw`]: Protocol::Macraw
    /// [`Open`]: SocketCommand::Open
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
/// After W5500 accepts the command, the [`sn_cr`] register is automatically
/// cleared to `0x00`.
/// Even though [`sn_cr`] is cleared to `0x00`, the command
/// is still being processed.
/// To check whether the command is completed or not, check
/// [`sn_ir`] or [`sn_sr`].
///
/// [`sn_cr`]: crate::Registers::set_sn_cr
/// [`sn_ir`]: crate::Registers::sn_ir
/// [`sn_sr`]: crate::Registers::sn_sr
#[derive(Copy, Clone, Eq, PartialEq, Debug, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "ufmt", derive(ufmt::derive::uDebug))]
#[repr(u8)]
pub enum SocketCommand {
    /// The command register clears to this state once a command has been
    /// accepted.
    Accepted = 0x00,
    /// The socket is initialized and opened according to the protocol
    /// selected in [`sn_mr`].
    ///
    /// | [`sn_mr`]            | [`sn_sr`]                   |
    /// |----------------------|-----------------------------|
    /// | [`Protocol::Closed`] | -                           |
    /// | [`Protocol::Tcp`]    | [`SocketStatus::Init`]      |
    /// | [`Protocol::Udp`]    | [`SocketStatus::Udp`]       |
    /// | [`Protocol::Macraw`] | [`SocketStatus::Macraw`]    |
    ///
    /// [`sn_mr`]: crate::Registers::sn_mr
    /// [`sn_sr`]: crate::Registers::sn_sr
    Open = 0x01,
    /// Operate the socket as a TCP server.
    ///
    /// This will change the socket state from [`Init`] to [`Listen`],
    /// and the socket will listen for a
    /// connection-request (SYN packet) from any TCP client.
    ///
    /// When a TCP client connection request is successfully established,
    /// the socket state changes from [`Listen`] to
    /// [`Established`] and the [`CON`] socket interrupt is raised.
    ///
    /// When a TCP client connection request fails the [`TIMEOUT`] socket
    /// interrupt is set and the
    /// socket status changes to [`Closed`].
    ///
    /// Only valid in [`Tcp`] mode.
    ///
    /// [`Closed`]: SocketStatus::Closed
    /// [`CON`]: crate::SocketInterrupt::con_raised
    /// [`Established`]: SocketStatus::Established
    /// [`Init`]: SocketStatus::Init
    /// [`Listen`]: SocketStatus::Listen
    /// [`Tcp`]: Protocol::Tcp
    /// [`TIMEOUT`]: crate::SocketInterrupt::timeout_raised
    Listen = 0x02,
    /// Connect to a TCP server.
    ///
    /// A connect-request (SYN packet) is sent to the TCP server configured by
    /// the IPv4 address and port set with [`set_sn_dest`].
    ///
    /// If the connect-request is successful, the socket state changes to
    /// [`Established`] and the [`CON`] socket interrupt is raised.
    ///
    /// The connect-request fails in the following three cases:
    /// 1. When a ARP<sub>TO</sub> occurs ([`timeout_raised`]) because the
    ///    destination hardware address is not acquired through the
    ///    ARP-process.
    /// 2. When a SYN/ACK packet is not received within the TCP timeout duration
    ///    set by [`rcr`] and [`rtr`] ([`timeout_raised`]).
    /// 3. When a RST packet is received instead of a SYN/ACK packet.
    ///
    /// In these cases the socket state changes to [`Closed`].
    ///
    /// Only valid in [`Tcp`] mode when acting as a TCP client.
    ///
    /// [`Closed`]: SocketStatus::Closed
    /// [`CON`]: crate::SocketInterrupt::con_raised
    /// [`Established`]: SocketStatus::Established
    /// [`rcr`]: crate::Registers::rcr
    /// [`rtr`]: crate::Registers::rtr
    /// [`set_sn_dest`]: crate::Registers::set_sn_dest
    /// [`Tcp`]: Protocol::Tcp
    /// [`timeout_raised`]: crate::SocketInterrupt::timeout_raised
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
    /// the socket state changes to [`Closed`].
    /// Otherwise, TCP timeout occurs
    /// ([`timeout_raised`]) and then
    /// the socket state changes to [`Closed`].
    ///
    /// If the [`Close`] command is used instead of
    /// [`Disconnect`], the socket state is changes to
    /// [`Closed`] without the disconnect process.
    ///
    /// If a RST packet is received from a peer during communication the socket
    /// status is unconditionally changed to [`Closed`].
    ///
    /// Only valid in [`Tcp`] mode.
    ///
    /// [`Disconnect`]: SocketCommand::Disconnect
    /// [`Close`]: SocketCommand::Close
    /// [`Closed`]: SocketStatus::Closed
    /// [`Tcp`]: Protocol::Tcp
    /// [`timeout_raised`]: crate::SocketInterrupt::timeout_raised
    Disconnect = 0x08,
    /// Close the socket.
    ///
    /// The socket status is changed to [`Closed`].
    ///
    /// [`Closed`]: SocketStatus::Closed
    Close = 0x10,
    /// Transmits all the data in the socket TX buffer.
    Send = 0x20,
    /// The basic operation is same as [`Send`].
    ///
    /// Normally [`Send`] transmits data after destination
    /// hardware address is acquired by the automatic ARP-process
    /// (Address Resolution Protocol).
    /// [`SendMac`] transmits data without the automatic
    /// ARP-process.
    /// In this case, the destination hardware address is acquired from
    /// [`sn_dhar`] configured by the host, instead of the ARP
    /// process.
    ///
    /// Only valid in [`Udp`] mode.
    ///
    /// [`Send`]: SocketCommand::Send
    /// [`SendMac`]: SocketCommand::SendMac
    /// [`Udp`]: Protocol::Udp
    /// [`sn_dhar`]: crate::Registers::sn_dhar
    SendMac = 0x21,
    /// Sends a 1 byte keep-alive packet.
    ///
    /// If the peer cannot respond to the keep-alive packet during timeout
    /// time, the connection is terminated and the timeout interrupt will
    /// occur ([`timeout_raised`]).
    ///
    /// Only valid in [`Tcp`] mode.
    ///
    /// [`Tcp`]: Protocol::Tcp
    /// [`timeout_raised`]: crate::SocketInterrupt::timeout_raised
    SendKeep = 0x22,
    /// Completes the processing of the received data in socket RX buffer.
    ///
    /// See [`sn_rx_buf`] for an example.
    ///
    /// [`sn_rx_buf`]: crate::Registers::sn_rx_buf
    Recv = 0x40,
}
impl From<SocketCommand> for u8 {
    fn from(val: SocketCommand) -> u8 {
        val as u8
    }
}
impl TryFrom<u8> for SocketCommand {
    type Error = u8;
    fn try_from(val: u8) -> Result<Self, u8> {
        match val {
            x if x == Self::Accepted as u8 => Ok(Self::Accepted),
            x if x == Self::Open as u8 => Ok(Self::Open),
            x if x == Self::Listen as u8 => Ok(Self::Listen),
            x if x == Self::Connect as u8 => Ok(Self::Connect),
            x if x == Self::Disconnect as u8 => Ok(Self::Disconnect),
            x if x == Self::Close as u8 => Ok(Self::Close),
            x if x == Self::Send as u8 => Ok(Self::Send),
            x if x == Self::SendMac as u8 => Ok(Self::SendMac),
            x if x == Self::SendKeep as u8 => Ok(Self::SendKeep),
            x if x == Self::Recv as u8 => Ok(Self::Recv),
            _ => Err(val),
        }
    }
}

/// Socket protocol.
///
/// This is used by [`SocketMode::protocol`] method for the [`sn_mr`] register.
///
/// [`SocketMode::protocol`]: crate::SocketMode::protocol
/// [`sn_mr`]: crate::Registers::sn_mr
#[derive(Copy, Clone, Eq, PartialEq, Debug, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "ufmt", derive(ufmt::derive::uDebug))]
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
    /// MACRAW mode can only be used with [socket 0].
    ///
    /// [socket 0]: crate::Sn::Sn0
    Macraw = 0b0100,
}
impl Protocol {
    /// Convert a raw `u8` to an `Protocol`.
    ///
    /// Bit values that do not correspond to a protocol will be returned in the
    /// `Err` variant of the result.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::Protocol;
    ///
    /// assert_eq!(Protocol::from_raw(0b0000), Ok(Protocol::Closed));
    /// assert_eq!(Protocol::from_raw(0b0001), Ok(Protocol::Tcp));
    /// assert_eq!(Protocol::from_raw(0b0010), Ok(Protocol::Udp));
    /// assert_eq!(Protocol::from_raw(0b0100), Ok(Protocol::Macraw));
    /// assert_eq!(Protocol::from_raw(0b0101), Err(0b0101));
    /// ```
    pub const fn from_raw(val: u8) -> Result<Self, u8> {
        match val {
            x if x == Protocol::Closed as u8 => Ok(Protocol::Closed),
            x if x == Protocol::Tcp as u8 => Ok(Protocol::Tcp),
            x if x == Protocol::Udp as u8 => Ok(Protocol::Udp),
            x if x == Protocol::Macraw as u8 => Ok(Protocol::Macraw),
            _ => Err(val),
        }
    }
}
impl From<Protocol> for u8 {
    fn from(val: Protocol) -> u8 {
        val as u8
    }
}
impl Default for Protocol {
    fn default() -> Self {
        Self::Closed
    }
}
impl TryFrom<u8> for Protocol {
    type Error = u8;
    fn try_from(val: u8) -> Result<Self, u8> {
        Self::from_raw(val)
    }
}

/// PHY operation mode.
///
/// This is used by [`PhyCfg::opmdc`] method for the [`phycfgr`] register.
///
/// [`PhyCfg::opmdc`]: crate::PhyCfg::opmdc
/// [`phycfgr`]: crate::Registers::phycfgr
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "ufmt", derive(ufmt::derive::uDebug))]
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
impl OperationMode {
    /// Convert a raw `u8` to an `OperationMode`.
    ///
    /// Only the first 3 bits of the `u8` value are used.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::OperationMode;
    ///
    /// assert_eq!(
    ///     OperationMode::from_raw(0b000),
    ///     OperationMode::HalfDuplex10bt
    /// );
    /// assert_eq!(
    ///     OperationMode::from_raw(0b001),
    ///     OperationMode::FullDuplex10bt
    /// );
    /// assert_eq!(
    ///     OperationMode::from_raw(0b010),
    ///     OperationMode::HalfDuplex100bt
    /// );
    /// assert_eq!(
    ///     OperationMode::from_raw(0b011),
    ///     OperationMode::FullDuplex100bt
    /// );
    /// assert_eq!(
    ///     OperationMode::from_raw(0b100),
    ///     OperationMode::HalfDuplex100btAuto
    /// );
    /// assert_eq!(OperationMode::from_raw(0b110), OperationMode::PowerDown);
    /// assert_eq!(OperationMode::from_raw(0b111), OperationMode::Auto);
    /// ```
    pub const fn from_raw(val: u8) -> Self {
        match val & 0b111 {
            x if x == Self::HalfDuplex10bt as u8 => Self::HalfDuplex10bt,
            x if x == Self::FullDuplex10bt as u8 => Self::FullDuplex10bt,
            x if x == Self::HalfDuplex100bt as u8 => Self::HalfDuplex100bt,
            x if x == Self::FullDuplex100bt as u8 => Self::FullDuplex100bt,
            x if x == Self::HalfDuplex100btAuto as u8 => Self::HalfDuplex100btAuto,
            x if x == Self::PowerDown as u8 => Self::PowerDown,
            // x if x == Self::Auto as u8
            _ => Self::Auto,
        }
    }
}
impl From<OperationMode> for u8 {
    fn from(val: OperationMode) -> u8 {
        val as u8
    }
}
impl Default for OperationMode {
    fn default() -> Self {
        Self::Auto
    }
}

/// PHY link status.
///
/// This is used by [`PhyCfg::lnk`] method for the [`phycfgr`] register.
///
/// [`PhyCfg::lnk`]: crate::PhyCfg::lnk
/// [`phycfgr`]: crate::Registers::phycfgr
#[derive(Copy, Clone, Eq, PartialEq, Debug, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "ufmt", derive(ufmt::derive::uDebug))]
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
impl Default for LinkStatus {
    fn default() -> Self {
        LinkStatus::Down
    }
}

/// PHY speed status.
///
/// This is used by [`PhyCfg::spd`] method for the [`phycfgr`] register.
///
/// [`PhyCfg::spd`]: crate::PhyCfg::spd
/// [`phycfgr`]: crate::Registers::phycfgr
#[derive(Copy, Clone, Eq, PartialEq, Debug, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "ufmt", derive(ufmt::derive::uDebug))]
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
impl Default for SpeedStatus {
    fn default() -> Self {
        SpeedStatus::Mbps10
    }
}

/// PHY duplex status.
///
/// This is used by [`PhyCfg::dpx`] method for the [`phycfgr`] register.
///
/// [`PhyCfg::dpx`]: crate::PhyCfg::dpx
/// [`phycfgr`]: crate::Registers::phycfgr
#[derive(Copy, Clone, Eq, PartialEq, Debug, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "ufmt", derive(ufmt::derive::uDebug))]
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
impl Default for DuplexStatus {
    fn default() -> Self {
        DuplexStatus::Half
    }
}

/// RX and TX buffer sizes.
///
/// This is an argument of [`Registers::set_sn_rxbuf_size`] and
/// [`Registers::set_sn_txbuf_size`].
///
/// [`Registers::set_sn_txbuf_size`]: crate::Registers::set_sn_txbuf_size
/// [`Registers::set_sn_rxbuf_size`]: crate::Registers::set_sn_rxbuf_size
#[derive(Copy, Clone, Eq, PartialEq, PartialOrd, Ord, Hash, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "ufmt", derive(ufmt::derive::uDebug))]
#[repr(u8)]
#[allow(clippy::upper_case_acronyms)]
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
    /// Get the register value from a buffer size.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::BufferSize;
    ///
    /// assert_eq!(u8::from(BufferSize::KB0), 0);
    /// assert_eq!(u8::from(BufferSize::KB1), 1);
    /// assert_eq!(u8::from(BufferSize::KB2), 2);
    /// assert_eq!(u8::from(BufferSize::KB4), 4);
    /// assert_eq!(u8::from(BufferSize::KB8), 8);
    /// assert_eq!(u8::from(BufferSize::KB16), 16);
    /// ```
    fn from(val: BufferSize) -> u8 {
        val as u8
    }
}

impl TryFrom<u8> for BufferSize {
    type Error = u8;

    /// Get the buffer size given the register value.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::BufferSize;
    ///
    /// assert_eq!(BufferSize::try_from(0), Ok(BufferSize::KB0));
    /// assert_eq!(BufferSize::try_from(1), Ok(BufferSize::KB1));
    /// assert_eq!(BufferSize::try_from(2), Ok(BufferSize::KB2));
    /// assert_eq!(BufferSize::try_from(4), Ok(BufferSize::KB4));
    /// assert_eq!(BufferSize::try_from(8), Ok(BufferSize::KB8));
    /// assert_eq!(BufferSize::try_from(16), Ok(BufferSize::KB16));
    /// assert_eq!(BufferSize::try_from(17), Err(17));
    /// ```
    fn try_from(val: u8) -> Result<BufferSize, u8> {
        match val {
            x if x == BufferSize::KB0 as u8 => Ok(BufferSize::KB0),
            x if x == BufferSize::KB1 as u8 => Ok(BufferSize::KB1),
            x if x == BufferSize::KB2 as u8 => Ok(BufferSize::KB2),
            x if x == BufferSize::KB4 as u8 => Ok(BufferSize::KB4),
            x if x == BufferSize::KB8 as u8 => Ok(BufferSize::KB8),
            x if x == BufferSize::KB16 as u8 => Ok(BufferSize::KB16),
            _ => Err(val),
        }
    }
}

impl Default for BufferSize {
    /// Default buffer size.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::BufferSize;
    ///
    /// assert_eq!(BufferSize::default(), BufferSize::KB2);
    /// ```
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
