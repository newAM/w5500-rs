//! Register enumerations and structures.

use crate::specifiers::{
    DuplexStatus, LinkStatus, OperationMode, Protocol, SpeedStatus, ValueError,
};
use core::convert::TryFrom;

macro_rules! impl_u8_for {
    ($REG:ident) => {
        impl From<u8> for $REG {
            fn from(val: u8) -> $REG {
                $REG(val)
            }
        }

        impl From<$REG> for u8 {
            fn from(val: $REG) -> u8 {
                val.0
            }
        }
    };
}

/// Mode register (MR).
///
/// Used for software reset, and controlling modes of operation.
///
/// This is used by the [`crate::Registers::mr`] and
/// [`crate::Registers::set_mr`] methods.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Mode(u8);

impl Mode {
    /// Mode register reset value.
    pub const RESET: u8 = 0x00;

    /// Bit offset for the `RST` field.
    pub const RST_OFFSET: u8 = 7;
    /// Bit offset for the `WOL` field.
    pub const WOL_OFFSET: u8 = 5;
    /// Bit offset for the `PB` field.
    pub const PB_OFFSET: u8 = 4;
    /// Bit offset for the `PPPoE` field.
    pub const PPPOE_OFFSET: u8 = 3;
    /// Bit offset for the `FARP` field.
    pub const FARP_OFFSET: u8 = 1;

    /// Bit mask for the `RST` field.
    pub const RST_MASK: u8 = 1 << Self::RST_OFFSET;
    /// Bit mask for the `WOL` field.
    pub const WOL_MASK: u8 = 1 << Self::WOL_OFFSET;
    /// Bit mask for the `PB` field.
    pub const PB_MASK: u8 = 1 << Self::PB_OFFSET;
    /// Bit mask for the `PPPoE` field.
    pub const PPPOE_MASK: u8 = 1 << Self::PPPOE_OFFSET;
    /// Bit mask for the `FARP` field.
    pub const FARP_MASK: u8 = 1 << Self::FARP_OFFSET;

    /// Set the software reset bit to '1'.
    ///
    /// When reset all internal registers will be initialized.
    pub fn rst(&mut self) {
        self.0 |= Self::RST_MASK;
    }

    /// Wake on LAN.
    ///
    /// If WOL mode is enabled and the received magic packet over
    /// UDP has been normally processed, the interrupt pin (INTn) asserts to low.
    pub fn wol_enabled(&self) -> bool {
        self.0 & Self::WOL_MASK != 0
    }

    /// Enable wake on LAN.
    pub fn enable_wol(&mut self) {
        self.0 |= Self::WOL_MASK
    }

    /// Disable wake on LAN.
    pub fn disable_wol(&mut self) {
        self.0 &= !Self::WOL_MASK
    }

    /// Ping block mode.
    ///
    /// If enabled it blocks responses to ping requests.
    pub fn pb_enabled(&self) -> bool {
        self.0 & Self::PB_MASK != 0
    }

    /// Enable ping block.
    pub fn enable_pb(&mut self) {
        self.0 |= Self::PB_MASK
    }

    /// Disable ping block.
    pub fn disable_pb(&mut self) {
        self.0 &= !Self::PB_MASK
    }

    /// PPPoE mode.
    ///
    /// If you use ADSL this should be enabled.
    pub fn pppoe_enabled(&self) -> bool {
        self.0 & Self::PPPOE_MASK != 0
    }

    /// Enable PPPoE mode.
    pub fn enable_pppoe(&mut self) {
        self.0 |= Self::PPPOE_MASK
    }

    /// Disable PPPoE mode.
    pub fn disable_pppoe(&mut self) {
        self.0 &= !Self::PPPOE_MASK
    }

    /// Force ARP.
    ///
    /// When enabled it forces sending ARP request whenever data is sent.
    pub fn farp_enabled(&self) -> bool {
        self.0 & Self::FARP_MASK != 0
    }

    /// Enable force ARP.
    pub fn enable_farp(&mut self) {
        self.0 |= Self::FARP_MASK
    }

    /// Disable force ARP.
    pub fn disable_farp(&mut self) {
        self.0 &= !Self::FARP_MASK
    }
}
impl_u8_for!(Mode);

impl Default for Mode {
    fn default() -> Mode {
        Mode(Mode::RESET)
    }
}

/// Interrupt and interrupt mask register.
///
/// When used for interrupt masking:
/// * `false` = Interrupt is disabled.
/// * `true` = Interrupt is enabled.
///
/// When used for reading interrupt status:
/// * `false` = Interrupt is not raised.
/// * `true` = Interrupt is raised.
///
/// This is used by these methods:
/// * [`crate::Registers::ir`]
/// * [`crate::Registers::set_ir`]
/// * [`crate::Registers::imr`]
/// * [`crate::Registers::set_imr`]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Interrupt(u8);

impl Interrupt {
    /// Interrupt and interrupt mask reset value.
    pub const RESET: u8 = 0x00;

    /// Bit offset for the `CONFLICT` field.
    pub const CONFLICT_OFFSET: u8 = 7;
    /// Bit offset for the `UNREACH` field.
    pub const UNREACH_OFFSET: u8 = 6;
    /// Bit offset for the `PPPoE` field.
    pub const PPPOE_OFFSET: u8 = 5;
    /// Bit offset for the `MP` field.
    pub const MP_OFFSET: u8 = 4;

    /// Bit mask for the `CONFLICT` field.
    pub const CONFLICT_MASK: u8 = 1 << Self::CONFLICT_OFFSET;
    /// Bit mask for the `UNREACH` field.
    pub const UNREACH_MASK: u8 = 1 << Self::UNREACH_OFFSET;
    /// Bit mask for the `PPPoE` field.
    pub const PPPOE_MASK: u8 = 1 << Self::PPPOE_OFFSET;
    /// Bit mask for the `MP` field.
    pub const MP_MASK: u8 = 1 << Self::MP_OFFSET;

    /// Get the value of the IP conflict interrupt.
    ///
    /// This interrupt is set when our source IP is the same as the sender IP
    /// in the received ARP request.
    pub const fn conflict(&self) -> bool {
        self.0 & Self::CONFLICT_MASK != 0
    }

    /// Set the IP conflict bit.
    pub fn set_conflict(&mut self) {
        self.0 |= Self::CONFLICT_MASK
    }

    /// Clear the IP conflict bit.
    pub fn clear_conflict(&mut self) {
        self.0 &= !Self::CONFLICT_MASK
    }

    /// Get the destination unreachable interrupt.
    ///
    /// This interrupt is set when receiving the ICMP
    /// (Destination port unreachable) packet.
    ///
    /// When this interrupt is set destination information such as the IP
    /// address and port number may be checked with the corresponding UIPR &
    /// UPORTR.
    pub const fn unreach(&self) -> bool {
        self.0 & Self::UNREACH_MASK != 0
    }

    /// Set the destination unreachable bit.
    pub fn set_unreach(&mut self) {
        self.0 |= Self::UNREACH_MASK
    }

    /// Clear the destination unreachable bit.
    pub fn clear_unreach(&mut self) {
        self.0 &= !Self::UNREACH_MASK
    }

    /// Get the PPPoE connection close interrupt.
    ///
    /// This interrupt is set when PPPoE is disconnected during PPPoE.
    pub const fn pppoe(&self) -> bool {
        self.0 & Self::PPPOE_MASK != 0
    }

    /// Set the PPPoE connection close bit.
    pub fn set_pppoe(&mut self) {
        self.0 |= Self::PPPOE_MASK
    }

    /// Clear the PPPoE connection close bit.
    pub fn clear_pppoe(&mut self) {
        self.0 &= !Self::PPPOE_MASK
    }

    /// Get the magic packet interrupt.
    ///
    /// This interrupt is set when wake on LAN is enabled, and the magic packet
    /// is received.
    pub const fn mp(&self) -> bool {
        self.0 & Self::MP_MASK != 0
    }

    /// Set the magic packet bit.
    pub fn set_mp(&mut self) {
        self.0 |= Self::MP_MASK
    }

    /// Clear the magic packet bit.
    pub fn clear_mp(&mut self) {
        self.0 &= !Self::MP_MASK
    }
}
impl_u8_for!(Interrupt);

impl Default for Interrupt {
    fn default() -> Self {
        Interrupt(Interrupt::RESET)
    }
}

/// PHY configuration register (PHYCFGR).
///
/// Used for:
/// * PHY reset.
/// * PHY operation modes.
/// * PHY status.
///
/// This is used by the [`crate::Registers::phycfgr`] and
/// [`crate::Registers::set_phycfgr`] methods.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct PhyCfg(u8);
impl_u8_for!(PhyCfg);

impl PhyCfg {
    /// PHY configuration register reset value.
    pub const RESET: u8 = 0b10111000;

    /// Bit offset for the `RST` field.
    pub const RST_OFFSET: u8 = 7;
    /// Bit offset for the `OPMD` field.
    pub const OPMD_OFFSET: u8 = 6;
    /// Bit offset for the `OPMDC` field.
    pub const OPMDC_OFFSET: u8 = 3;
    /// Bit offset for the `DPX` field.
    pub const DPX_OFFSET: u8 = 2;
    /// Bit offset for the `SPD` field.
    pub const SPD_OFFSET: u8 = 1;
    /// Bit offset for the `LNK` field.
    pub const LNK_OFFSET: u8 = 0;

    /// Bit mask for the `RST` field.
    pub const RST_MASK: u8 = 1 << Self::RST_OFFSET;
    /// Bit mask for the `OPMD` field.
    pub const OPMD_MASK: u8 = 1 << Self::OPMD_OFFSET;
    /// Bit mask for the `OPMDC` field.
    pub const OPMDC_MASK: u8 = 0b111 << Self::OPMDC_OFFSET;
    /// Bit mask for the `DPX` field.
    pub const DPX_MASK: u8 = 1 << Self::DPX_OFFSET;
    /// Bit mask for the `SPD` field.
    pub const SPD_MASK: u8 = 1 << Self::SPD_OFFSET;
    /// Bit mask for the `LNK` field.
    pub const LNK_MASK: u8 = 1 << Self::LNK_OFFSET;

    /// Set the PHY reset bit to '0', resetting the PHY.
    pub fn rst(&mut self) {
        self.0 &= !Self::RST_MASK;
    }

    /// Get the PHY operation mode.
    ///
    /// * `true` configure PHY with software.
    /// * `false` (reset value) configure PHY with hardware.
    pub const fn opmd(&self) -> bool {
        self.0 & Self::OPMD_MASK != 0
    }

    /// Get the operation mode.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::{OperationMode, PhyCfg};
    ///
    /// let phy_cfg = PhyCfg::default();
    /// assert_eq!(phy_cfg.opmdc(), Ok(OperationMode::Auto));
    /// ```
    pub fn opmdc(&self) -> Result<OperationMode, ValueError> {
        OperationMode::try_from((self.0 & Self::OPMDC_MASK) >> Self::OPMDC_OFFSET)
    }

    /// Set the PHY operation mode.
    ///
    /// Setting this will also set [`PhyCfg::opmd`] to enable PHY configuration
    /// with the value in this register.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::{OperationMode, PhyCfg};
    ///
    /// let mut phy_cfg = PhyCfg::default();
    /// assert!(!phy_cfg.opmd());
    /// phy_cfg.set_opmdc(OperationMode::PowerDown);
    /// assert!(phy_cfg.opmd());
    /// assert_eq!(phy_cfg.opmdc(), Ok(OperationMode::PowerDown));
    /// phy_cfg.set_opmdc(OperationMode::Auto);
    /// assert_eq!(phy_cfg.opmdc(), Ok(OperationMode::Auto));
    /// ```
    pub fn set_opmdc(&mut self, mode: OperationMode) {
        self.0 |= Self::OPMD_MASK;
        self.0 &= !Self::OPMDC_MASK;
        self.0 |= u8::from(mode) << Self::OPMDC_OFFSET;
    }

    /// Get the duplex status.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::{DuplexStatus, PhyCfg};
    ///
    /// let phy_cfg = PhyCfg::default();
    /// assert_eq!(phy_cfg.dpx(), DuplexStatus::Half);
    /// ```
    pub fn dpx(&self) -> DuplexStatus {
        DuplexStatus::from(self.0 & Self::DPX_MASK == Self::DPX_MASK)
    }

    /// Get the speed status.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::{PhyCfg, SpeedStatus};
    ///
    /// let phy_cfg = PhyCfg::default();
    /// assert_eq!(phy_cfg.spd(), SpeedStatus::Mbps10);
    /// ```
    pub fn spd(&self) -> SpeedStatus {
        SpeedStatus::from(self.0 & Self::SPD_MASK == Self::SPD_MASK)
    }

    /// Get the link status.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::{LinkStatus, PhyCfg};
    ///
    /// let phy_cfg = PhyCfg::default();
    /// assert_eq!(phy_cfg.lnk(), LinkStatus::Down);
    /// ```
    pub fn lnk(&self) -> LinkStatus {
        LinkStatus::from(self.0 & Self::LNK_MASK == Self::LNK_MASK)
    }
}

impl Default for PhyCfg {
    fn default() -> Self {
        Self(Self::RESET)
    }
}

impl ::core::fmt::Display for PhyCfg {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        write!(
            f,
            r#"PHY configuration:
Link status: {:?}
Speed status: {:?}
Duplex status: {:?}
Operation mode: {:?}
"#,
            self.lnk(),
            self.spd(),
            self.dpx(),
            self.opmdc()
        )
    }
}

/// Socket Mode Register (Sn_MR).
///
/// This is used by the [`crate::Registers::sn_mr`] and
/// [`crate::Registers::set_sn_mr`] methods.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct SocketMode(u8);
impl_u8_for!(SocketMode);

impl Default for SocketMode {
    fn default() -> Self {
        Self(Self::RESET)
    }
}

impl SocketMode {
    /// Reset value of the socket mode register.
    pub const RESET: u8 = 0x00;

    /// Bit offset for the `MULTI` field.
    pub const MULTI_OFFSET: u8 = 7;
    /// Bit offset for the `MFEN` field.
    pub const MFEN_OFFSET: u8 = 7;
    /// Bit offset for the `BCASTB` field.
    pub const BCASTB_OFFSET: u8 = 6;
    /// Bit offset for the `ND` field.
    pub const ND_OFFSET: u8 = 5;
    /// Bit offset for the `MC` field.
    pub const MC_OFFSET: u8 = 5;
    /// Bit offset for the `MMB` field.
    pub const MMB_OFFSET: u8 = 5;
    /// Bit offset for the `UCASTB` field.
    pub const UCASTB_OFFSET: u8 = 4;
    /// Bit offset for the `MIP6B` field.
    pub const MIP6B_OFFSET: u8 = 4;

    /// Bit mask for the `MULTI` field.
    pub const MULTI_MASK: u8 = 1 << Self::MULTI_OFFSET;
    /// Bit mask for the `MFEN` field.
    pub const MFEN_MASK: u8 = 1 << Self::MFEN_OFFSET;
    /// Bit mask for the `BCASTB` field.
    pub const BCASTB_MASK: u8 = 1 << Self::BCASTB_OFFSET;
    /// Bit mask for the `ND` field.
    pub const ND_MASK: u8 = 1 << Self::ND_OFFSET;
    /// Bit mask for the `MC` field.
    pub const MC_MASK: u8 = 1 << Self::MC_OFFSET;
    /// Bit mask for the `MMB` field.
    pub const MMB_MASK: u8 = 1 << Self::MMB_OFFSET;
    /// Bit mask for the `UCASTB` field.
    pub const UCASTB_MASK: u8 = 1 << Self::UCASTB_OFFSET;
    /// Bit mask for the `MIP6B` field.
    pub const MIP6B_MASK: u8 = 1 << Self::MIP6B_OFFSET;
    /// Bit mask for the `PROTOCOL` field.
    pub const PROTOCOL_MASK: u8 = 0xF;

    /// Get the protocol.
    ///
    /// This returns a `ValueError` if the protocol bits do not match a valid
    /// protocol.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::{Protocol, SocketMode};
    ///
    /// let mode: SocketMode = SocketMode::default();
    /// assert_eq!(mode.protocol(), Ok(Protocol::Closed));
    /// ```
    pub fn protocol(&self) -> Result<Protocol, ValueError> {
        Protocol::try_from(self.0 & Self::PROTOCOL_MASK)
    }

    /// Set the protocol.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::{Protocol, SocketMode};
    ///
    /// let mut mode: SocketMode = SocketMode::default();
    /// mode.set_protocol(Protocol::Tcp);
    /// assert_eq!(mode.protocol(), Ok(Protocol::Tcp));
    /// ```
    pub fn set_protocol(&mut self, protocol: Protocol) {
        self.0 = (self.0 & 0xF0) | ((protocol as u8) & 0xF)
    }

    /// Multicasting.
    ///
    /// This applies only for a socket with the UDP protocol.
    ///
    /// To use multicasting [`crate::Registers::sn_dipr`] and [`crate::Registers::sn_dport`]
    /// should be configured with the multicast group IP and port number
    /// before the socket is opened.
    pub const fn multi_enabled(&self) -> bool {
        self.0 & Self::MULTI_MASK != 0
    }

    /// Enable multicasting.
    ///
    /// See [`SocketMode::multi_enabled`] for more information.
    pub fn enable_multi(&mut self) {
        self.0 |= Self::MULTI_MASK
    }

    /// Disable multicasting.
    ///
    /// See [`SocketMode::multi_enabled`] for more information.
    pub fn disable_multi(&mut self) {
        self.0 &= !Self::MULTI_MASK
    }

    /// MAC filter.
    ///
    /// This applies only for a socket with the MACRAW protocol.
    ///
    /// When enabled the W5500 can only receive broadcasting packets sent to
    /// itself.
    /// When disabled the W5500 can receive all packets.
    /// If you want to implement a hybrid TCP/IP stack it is recommended that
    /// this is enabled for reducing host overhead to process all the received
    /// packets.
    pub const fn mfen_enabled(&self) -> bool {
        self.0 & Self::MFEN_MASK != 0
    }

    /// Enable MAC filter.
    ///
    /// See [`SocketMode::mfen_enabled`] for more information.
    pub fn enable_mfen(&mut self) {
        self.0 |= Self::MFEN_MASK
    }

    /// Disable MAC filter.
    ///
    /// See [`SocketMode::mfen_enabled`] for more information.
    pub fn disable_mfen(&mut self) {
        self.0 &= !Self::MFEN_MASK
    }

    /// Broadcast blocking.
    ///
    /// This applies only for a socket with the MACRAW or UDP protocol.
    pub const fn bcastb_enabled(&self) -> bool {
        self.0 & Self::BCASTB_MASK != 0
    }

    /// Enable broadcast blocking.
    ///
    /// See [`SocketMode::bcastb_enabled`] for more information.
    pub fn enable_bcastb(&mut self) {
        self.0 |= Self::BCASTB_MASK
    }

    /// Disable broadcast blocking.
    ///
    /// See [`SocketMode::bcastb_enabled`] for more information.
    pub fn disable_bcastb(&mut self) {
        self.0 &= !Self::BCASTB_MASK
    }

    /// Use no delayed ACK.
    ///
    /// This applies only for a socket with the TCP protocol.
    ///
    /// When enabled the ACK packet is sent without delay as soon as a data
    /// packet is received from a peer.
    /// When disabled the ACK packet is sent after waiting for the time
    /// configured by [`crate::Registers::rtr`].
    pub const fn nd_enabled(&self) -> bool {
        self.0 & Self::ND_MASK != 0
    }

    /// Disable no delayed ACK.
    ///
    /// See [`SocketMode::nd_enabled`] for more information.
    pub fn disable_nd(&mut self) {
        self.0 &= !Self::ND_MASK
    }

    /// Enable no delayed ACK.
    ///
    /// See [`SocketMode::nd_enabled`] for more information.
    pub fn enable_nd(&mut self) {
        self.0 |= Self::ND_MASK
    }

    /// Multicast IGMP version.
    ///
    /// This applies only for a socket with the UDP protocol.
    ///
    /// This field configures the version for IGMP messages (join/leave/report).
    ///
    /// * `false` IGMP version 2
    /// * `true` IGMP version 1
    pub const fn mc(&self) -> bool {
        self.0 & Self::MC_MASK != 0
    }

    /// Set IGMP version 1.
    ///
    /// See [`SocketMode::mc`] for more information.
    pub fn set_igmp_v1(&mut self) {
        self.0 |= Self::MC_MASK
    }

    /// Set IGMP version 2.
    ///
    /// See [`SocketMode::mc`] for more information.
    pub fn set_igmp_v2(&mut self) {
        self.0 &= !Self::MC_MASK
    }

    /// Multicast blocking.
    ///
    /// This applies only for a socket with the MACRAW protocol.
    pub const fn mmb_enabled(&self) -> bool {
        self.0 & Self::MMB_MASK != 0
    }

    /// Enable multicast blocking.
    ///
    /// See [`SocketMode::mmb_enabled`] for more information.
    pub fn enable_mmb(&mut self) {
        self.0 |= Self::MMB_MASK
    }

    /// Disable multicast blocking.
    ///
    /// See [`SocketMode::mmb_enabled`] for more information.
    pub fn disable_mmb(&mut self) {
        self.0 &= !Self::MMB_MASK
    }

    /// Unicast blocking enabled.
    ///
    /// This applies only for a socket with the UDP protocol.
    pub const fn ucastb_enabled(&self) -> bool {
        self.0 & Self::UCASTB_MASK != 0
    }

    /// Enable unicast blocking.
    ///
    /// See [`SocketMode::ucastb_enabled`] for more information.
    pub fn enable_ucastb(&mut self) {
        self.0 |= Self::UCASTB_MASK
    }

    /// Disable unicast blocking.
    ///
    /// See [`SocketMode::ucastb_enabled`] for more information.
    pub fn disable_ucastb(&mut self) {
        self.0 &= !Self::UCASTB_MASK
    }

    /// IPV6 packet blocking.
    ///
    /// This applies only for a socket with the MACRAW protocol.
    pub const fn mip6b_enabled(&self) -> bool {
        self.0 & Self::MIP6B_MASK != 0
    }

    /// Enable unicast blocking.
    ///
    /// See [`SocketMode::mip6b_enabled`] for more information.
    pub fn enable_mip6b(&mut self) {
        self.0 |= Self::UCASTB_MASK
    }

    /// Disable unicast blocking.
    ///
    /// See [`SocketMode::mip6b_enabled] for more information.
    pub fn disable_mip6b(&mut self) {
        self.0 &= !Self::UCASTB_MASK
    }
}

/// Socket Interrupt Register.
///
/// Indicated the socket status, such as connection, termination,
/// receiving data, and timeout.
///
/// This is used by the [`crate::Registers::sn_ir`] and
/// [`crate::Registers::set_sn_ir`] methods.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct SocketInterrupt(u8);
impl_u8_for!(SocketInterrupt);

impl Default for SocketInterrupt {
    fn default() -> Self {
        Self(Self::RESET)
    }
}

impl SocketInterrupt {
    /// Socket interrupt status register (Sn_IR) reset value.
    pub const RESET: u8 = 0x00;

    /// Bit offset for the `CON` field.
    pub const CON_OFFSET: u8 = 0;
    /// Bit offset for the `DISCON` field.
    pub const DISCON_OFFSET: u8 = 1;
    /// Bit offset for the `RECV` field.
    pub const RECV_OFFSET: u8 = 2;
    /// Bit offset for the `TIMEOUT` field.
    pub const TIMEOUT_OFFSET: u8 = 3;
    /// Bit offset for the `SENDOK` field.
    pub const SENDOK_OFFSET: u8 = 4;

    /// Bit mask for the `CON` field.
    pub const CON_MASK: u8 = 1 << Self::CON_OFFSET;
    /// Bit mask for the `DISCON` field.
    pub const DISCON_MASK: u8 = 1 << Self::DISCON_OFFSET;
    /// Bit mask for the `RECV` field.
    pub const RECV_MASK: u8 = 1 << Self::RECV_OFFSET;
    /// Bit mask for the `TIMEOUT` field.
    pub const TIMEOUT_MASK: u8 = 1 << Self::TIMEOUT_OFFSET;
    /// Bit mask for the `SENDOK` field.
    pub const SENDOK_MASK: u8 = 1 << Self::SENDOK_OFFSET;

    /// Get the value of the `CON` interrupt.
    ///
    /// This is issued once when the connection with the peer is successful.
    pub const fn con_raised(&self) -> bool {
        self.0 & Self::CON_MASK != 0
    }

    /// Clear the `CON` interrupt by writing `1`.
    pub fn clear_con(&mut self) {
        self.0 |= Self::CON_MASK
    }

    /// Get the value of the `DISCON` interrupt.
    ///
    /// This is issued when FIN or FIN/ACK packet is received from a peer.
    pub const fn discon_raised(&self) -> bool {
        self.0 & Self::DISCON_MASK != 0
    }

    /// Clear the `DISCON` interrupt by writing `1`.
    pub fn clear_discon(&mut self) {
        self.0 |= Self::CON_MASK
    }

    /// Get the value of the `RECV` interrupt.
    ///
    /// This is issued whenever data is received from a peer.
    pub const fn recv_raised(&self) -> bool {
        self.0 & Self::RECV_MASK != 0
    }

    /// Clear the `RECV` interrupt by writing `1`.
    pub fn clear_recv(&mut self) {
        self.0 |= Self::RECV_MASK
    }

    /// Get the value of the `TIMEOUT` interrupt.
    ///
    /// This is issued when ARP<sub>TO</sub> or TCP<sub>TO</sub> occurs.
    pub const fn timeout_raised(&self) -> bool {
        self.0 & Self::TIMEOUT_MASK != 0
    }

    /// Clear the `TIMEOUT` interrupt by writing `1`.
    pub fn clear_timeout(&mut self) {
        self.0 |= Self::TIMEOUT_MASK
    }

    /// Get the value of the `SENDOK` interrupt.
    ///
    /// This is issued when SEND command is completed.
    pub fn sendok_raised(&self) -> bool {
        self.0 & Self::SENDOK_MASK != 0
    }

    /// Clear the `SENDOK` interrupt by writing `1`.
    pub fn clear_sendok(&mut self) {
        self.0 |= Self::SENDOK_MASK
    }
}

/// Socket Interrupt Mask Register.
///
/// This is used by the [`crate::Registers::sn_imr`] and
/// [`crate::Registers::set_sn_imr`] methods.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct SocketInterruptMask(u8);
impl_u8_for!(SocketInterruptMask);

impl Default for SocketInterruptMask {
    fn default() -> Self {
        Self(Self::RESET)
    }
}

impl SocketInterruptMask {
    /// Socket interrupt mask register (Sn_IMR) reset value.
    pub const RESET: u8 = 0xFF;

    /// Mask all socket interrupts.
    pub const ALL_MASKED: SocketInterruptMask = SocketInterruptMask(0xE0);

    /// Check if the `CON` interrupt is masked.
    pub const fn con_masked(&self) -> bool {
        self.0 & SocketInterrupt::CON_MASK != 0
    }

    /// Unmask the `CON` interrupt.
    pub fn unmask_con(&mut self) {
        self.0 |= SocketInterrupt::CON_MASK
    }

    /// Mask the `CON` interrupt.
    pub fn mask_con(&mut self) {
        self.0 &= !SocketInterrupt::CON_MASK
    }

    /// Check if the `DISCON` interrupt is masked.
    pub const fn discon_masked(&self) -> bool {
        self.0 & SocketInterrupt::DISCON_MASK != 0
    }

    /// Unmask the `DISCON` interrupt.
    pub fn unmask_discon(&mut self) {
        self.0 |= SocketInterrupt::CON_MASK
    }

    /// Mask the `DISCON` interrupt.
    pub fn mask_discon(&mut self) {
        self.0 &= !SocketInterrupt::CON_MASK
    }

    /// Check if the `RECV` interrupt is masked.
    pub const fn recv_masked(&self) -> bool {
        self.0 & SocketInterrupt::RECV_MASK != 0
    }

    /// Unmask the `RECV` interrupt.
    pub fn unmask_recv(&mut self) {
        self.0 |= SocketInterrupt::RECV_MASK
    }

    /// Mask the `RECV` interrupt.
    pub fn mask_recv(&mut self) {
        self.0 &= !SocketInterrupt::RECV_MASK
    }

    /// Check if the `TIMEOUT` interrupt is masked.
    pub const fn timeout_masked(&self) -> bool {
        self.0 & SocketInterrupt::TIMEOUT_MASK != 0
    }

    /// Unmask the `TIMEOUT` interrupt.
    pub fn unmask_timeout(&mut self) {
        self.0 |= SocketInterrupt::TIMEOUT_MASK
    }

    /// Mask the `TIMEOUT` interrupt.
    pub fn mask_timeout(&mut self) {
        self.0 &= !SocketInterrupt::TIMEOUT_MASK
    }

    /// Check if the `SENDOK` interrupt is masked.
    pub fn sendok_masked(&self) -> bool {
        self.0 & SocketInterrupt::SENDOK_MASK != 0
    }

    /// Unmask the `SENDOK` interrupt.
    pub fn unmask_sendok(&mut self) {
        self.0 |= SocketInterrupt::SENDOK_MASK
    }

    /// Mask the `SENDOK` interrupt.
    pub fn mask_sendok(&mut self) {
        self.0 &= !SocketInterrupt::SENDOK_MASK
    }
}
