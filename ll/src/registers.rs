//! Register structures.

use crate::specifiers::{DuplexStatus, LinkStatus, OperationMode, Protocol, SpeedStatus};

macro_rules! impl_boilerplate_for {
    ($REG:ident) => {
        impl From<u8> for $REG {
            fn from(val: u8) -> Self {
                Self(val)
            }
        }

        impl From<$REG> for u8 {
            fn from(val: $REG) -> u8 {
                val.0
            }
        }

        impl Default for $REG {
            fn default() -> Self {
                Self::DEFAULT
            }
        }
    };
}

/// Mode register (MR).
///
/// Used for software reset, and controlling modes of operation.
///
/// This is used by the [`Registers::mr`] and [`Registers::set_mr`] methods.
///
/// [`Registers::mr`]: crate::Registers::mr
/// [`Registers::set_mr`]: crate::Registers::set_mr
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Mode(u8);
impl_boilerplate_for!(Mode);

impl Mode {
    /// Mode register reset value.
    pub const RESET: u8 = 0x00;

    /// Default value.
    ///
    /// This is the same as `default`, but as a `const` value.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::Mode;
    ///
    /// assert_eq!(Mode::DEFAULT, Mode::default());
    /// ```
    pub const DEFAULT: Self = Self(Self::RESET);

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

    /// Set the software reset bit to `1`.
    ///
    /// When reset all internal registers will be initialized.
    #[must_use = "rst returns a modified Mode"]
    pub const fn rst(mut self) -> Self {
        self.0 |= Self::RST_MASK;
        self
    }

    /// Wake on LAN.
    ///
    /// If WOL mode is enabled and the received magic packet over
    /// UDP has been normally processed, the interrupt pin (INTn) asserts to low.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::Mode;
    ///
    /// let mr: Mode = Mode::DEFAULT;
    /// assert!(!mr.wol_enabled());
    /// let mr: Mode = mr.enable_wol();
    /// assert!(mr.wol_enabled());
    /// let mr: Mode = mr.disable_wol();
    /// assert!(!mr.wol_enabled());
    /// ```
    pub const fn wol_enabled(&self) -> bool {
        self.0 & Self::WOL_MASK != 0
    }

    /// Enable wake on LAN.
    #[must_use = "enable_wol returns a modified Mode"]
    pub const fn enable_wol(mut self) -> Self {
        self.0 |= Self::WOL_MASK;
        self
    }

    /// Disable wake on LAN.
    #[must_use = "disable_wol returns a modified Mode"]
    pub const fn disable_wol(mut self) -> Self {
        self.0 &= !Self::WOL_MASK;
        self
    }

    /// Ping block mode.
    ///
    /// If enabled it blocks responses to ping requests.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::Mode;
    ///
    /// let mr: Mode = Mode::DEFAULT;
    /// assert!(!mr.pb_enabled());
    /// let mr: Mode = mr.enable_pb();
    /// assert!(mr.pb_enabled());
    /// let mr: Mode = mr.disable_pb();
    /// assert!(!mr.pb_enabled());
    /// ```
    pub const fn pb_enabled(&self) -> bool {
        self.0 & Self::PB_MASK != 0
    }

    /// Enable ping block.
    #[must_use = "enable_pb returns a modified Mode"]
    pub const fn enable_pb(mut self) -> Self {
        self.0 |= Self::PB_MASK;
        self
    }

    /// Disable ping block.
    #[must_use = "disable_pb returns a modified Mode"]
    pub const fn disable_pb(mut self) -> Self {
        self.0 &= !Self::PB_MASK;
        self
    }

    /// PPPoE mode.
    ///
    /// If you use ADSL this should be enabled.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::Mode;
    ///
    /// let mr: Mode = Mode::DEFAULT;
    /// assert!(!mr.pppoe_enabled());
    /// let mr: Mode = mr.enable_pppoe();
    /// assert!(mr.pppoe_enabled());
    /// let mr: Mode = mr.disable_pppoe();
    /// assert!(!mr.pppoe_enabled());
    /// ```
    pub const fn pppoe_enabled(&self) -> bool {
        self.0 & Self::PPPOE_MASK != 0
    }

    /// Enable PPPoE mode.
    #[must_use = "enable_pppoe returns a modified Mode"]
    pub const fn enable_pppoe(mut self) -> Self {
        self.0 |= Self::PPPOE_MASK;
        self
    }

    /// Disable PPPoE mode.
    #[must_use = "disable_pppoe returns a modified Mode"]
    pub const fn disable_pppoe(mut self) -> Self {
        self.0 &= !Self::PPPOE_MASK;
        self
    }

    /// Force ARP.
    ///
    /// When enabled it forces sending ARP request whenever data is sent.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::Mode;
    ///
    /// let mr: Mode = Mode::DEFAULT;
    /// assert!(!mr.farp_enabled());
    /// let mr: Mode = mr.enable_farp();
    /// assert!(mr.farp_enabled());
    /// let mr: Mode = mr.disable_farp();
    /// assert!(!mr.farp_enabled());
    /// ```
    pub const fn farp_enabled(&self) -> bool {
        self.0 & Self::FARP_MASK != 0
    }

    /// Enable force ARP.
    #[must_use = "enable_farp returns a modified Mode"]
    pub const fn enable_farp(mut self) -> Self {
        self.0 |= Self::FARP_MASK;
        self
    }

    /// Disable force ARP.
    #[must_use = "disable_farp returns a modified Mode"]
    pub const fn disable_farp(mut self) -> Self {
        self.0 &= !Self::FARP_MASK;
        self
    }
}

impl ::core::fmt::Display for Mode {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("Mode")
            .field("wol_enabled", &self.wol_enabled())
            .field("pb_enabled", &self.pb_enabled())
            .field("pppoe_enabled", &self.pppoe_enabled())
            .field("farp_enabled", &self.farp_enabled())
            .finish()
    }
}

/// Interrupt and interrupt mask register (IR and IMR).
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
/// * [`Registers::ir`]
/// * [`Registers::set_ir`]
/// * [`Registers::imr`]
/// * [`Registers::set_imr`]
///
/// [`Registers::ir`]: crate::Registers::ir
/// [`Registers::set_ir`]: crate::Registers::set_ir
/// [`Registers::imr`]: crate::Registers::imr
/// [`Registers::set_imr`]: crate::Registers::set_imr
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Interrupt(u8);
impl_boilerplate_for!(Interrupt);

impl Interrupt {
    /// Interrupt and interrupt mask reset value.
    pub const RESET: u8 = 0x00;

    /// Default value.
    ///
    /// This is the same as `default`, but as a `const` value.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::Interrupt;
    ///
    /// assert_eq!(Interrupt::DEFAULT, Interrupt::default());
    /// ```
    pub const DEFAULT: Self = Self(Self::RESET);

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
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::Interrupt;
    ///
    /// let ir: Interrupt = Interrupt::DEFAULT;
    /// assert!(!ir.conflict());
    /// let ir: Interrupt = ir.set_conflict();
    /// assert!(ir.conflict());
    /// let ir: Interrupt = ir.clear_conflict();
    /// assert!(!ir.conflict());
    /// ```
    pub const fn conflict(&self) -> bool {
        self.0 & Self::CONFLICT_MASK != 0
    }

    /// Set the IP conflict bit.
    #[must_use = "set_conflict returns a modified Interrupt"]
    pub const fn set_conflict(mut self) -> Self {
        self.0 |= Self::CONFLICT_MASK;
        self
    }

    /// Clear the IP conflict bit.
    #[must_use = "clear_conflict returns a modified Interrupt"]
    pub const fn clear_conflict(mut self) -> Self {
        self.0 &= !Self::CONFLICT_MASK;
        self
    }

    /// Get the destination unreachable interrupt.
    ///
    /// This interrupt is set when receiving the ICMP
    /// (destination port unreachable) packet.
    ///
    /// When this interrupt is set destination information such as the IP
    /// address and port number may be checked with the corresponding [UIPR] and
    /// [UPORTR] registers.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::Interrupt;
    ///
    /// let ir: Interrupt = Interrupt::DEFAULT;
    /// assert!(!ir.unreach());
    /// let ir: Interrupt = ir.set_unreach();
    /// assert!(ir.unreach());
    /// let ir: Interrupt = ir.clear_unreach();
    /// assert!(!ir.unreach());
    /// ```
    ///
    /// [UIPR]: crate::Registers::uipr
    /// [UPORTR]: crate::Registers::uportr
    pub const fn unreach(&self) -> bool {
        self.0 & Self::UNREACH_MASK != 0
    }

    /// Set the destination unreachable bit.
    #[must_use = "set_unreach returns a modified Interrupt"]
    pub const fn set_unreach(mut self) -> Self {
        self.0 |= Self::UNREACH_MASK;
        self
    }

    /// Clear the destination unreachable bit.
    #[must_use = "clear_unreach returns a modified Interrupt"]
    pub const fn clear_unreach(mut self) -> Self {
        self.0 &= !Self::UNREACH_MASK;
        self
    }

    /// Get the PPPoE connection close interrupt.
    ///
    /// This interrupt is set when PPPoE is disconnected during PPPoE.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::Interrupt;
    ///
    /// let ir: Interrupt = Interrupt::DEFAULT;
    /// assert!(!ir.pppoe());
    /// let ir: Interrupt = ir.set_pppoe();
    /// assert!(ir.pppoe());
    /// let ir: Interrupt = ir.clear_pppoe();
    /// assert!(!ir.pppoe());
    /// ```
    pub const fn pppoe(&self) -> bool {
        self.0 & Self::PPPOE_MASK != 0
    }

    /// Set the PPPoE connection close bit.
    #[must_use = "set_pppoe returns a modified Interrupt"]
    pub const fn set_pppoe(mut self) -> Self {
        self.0 |= Self::PPPOE_MASK;
        self
    }

    /// Clear the PPPoE connection close bit.
    #[must_use = "clear_pppoe returns a modified Interrupt"]
    pub const fn clear_pppoe(mut self) -> Self {
        self.0 &= !Self::PPPOE_MASK;
        self
    }

    /// Get the magic packet interrupt.
    ///
    /// This interrupt is set when wake on LAN is enabled, and the magic packet
    /// is received.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::Interrupt;
    ///
    /// let ir: Interrupt = Interrupt::DEFAULT;
    /// assert!(!ir.mp());
    /// let ir: Interrupt = ir.set_mp();
    /// assert!(ir.mp());
    /// let ir: Interrupt = ir.clear_mp();
    /// assert!(!ir.mp());
    /// ```
    pub const fn mp(&self) -> bool {
        self.0 & Self::MP_MASK != 0
    }

    /// Set the magic packet bit.
    #[must_use = "set_mp returns a modified Interrupt"]
    pub const fn set_mp(mut self) -> Self {
        self.0 |= Self::MP_MASK;
        self
    }

    /// Clear the magic packet bit.
    #[must_use = "clear_mp returns a modified Interrupt"]
    pub const fn clear_mp(mut self) -> Self {
        self.0 &= !Self::MP_MASK;
        self
    }
}

impl ::core::fmt::Display for Interrupt {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("Interrupt")
            .field("conflict", &self.conflict())
            .field("unreach", &self.unreach())
            .field("pppoe", &self.pppoe())
            .field("mp", &self.mp())
            .finish()
    }
}

/// PHY configuration register (PHYCFGR).
///
/// Used for:
/// * PHY reset.
/// * PHY operation modes.
/// * PHY status.
///
/// This is used by the [`Registers::phycfgr`] and
/// [`Registers::set_phycfgr`] methods.
///
/// [`Registers::phycfgr`]: crate::Registers::phycfgr
/// [`Registers::set_phycfgr`]: crate::Registers::set_phycfgr
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct PhyCfg(u8);
impl_boilerplate_for!(PhyCfg);

impl PhyCfg {
    /// PHY configuration register reset value.
    pub const RESET: u8 = 0b10111000;

    /// Default value.
    ///
    /// This is the same as `default`, but as a `const` value.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::PhyCfg;
    ///
    /// assert_eq!(PhyCfg::DEFAULT, PhyCfg::default());
    /// ```
    pub const DEFAULT: Self = Self(Self::RESET);

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

    /// Set the PHY reset bit to `0`, resetting the PHY.
    #[must_use = "rst returns a modified PhyCfg"]
    pub const fn rst(mut self) -> Self {
        self.0 &= !Self::RST_MASK;
        self
    }

    /// Get the PHY operation mode.
    ///
    /// * `true` configure PHY with software.
    /// * `false` (reset value) configure PHY with hardware.
    pub const fn opmd(&self) -> bool {
        self.0 & Self::OPMD_MASK != 0
    }

    /// Enable hardware configuration of the PHY operation mode.
    ///
    /// This uses the PMODE pins to select the PHY operation mode.
    ///
    /// | PMODE\[2\] | PMODE\[1\] | PMODE\[0\] | Description                                  |
    /// |------------|------------|------------|----------------------------------------------|
    /// | 0          | 0          | 0          | 10BT Half-duplex, Auto-negotiation disabled  |
    /// | 0          | 0          | 1          | 10BT Full-duplex, Auto-negotiation disabled  |
    /// | 0          | 1          | 0          | 100BT Half-duplex, Auto-negotiation disabled |
    /// | 0          | 1          | 1          | 100BT Full-duplex, Auto-negotiation disabled |
    /// | 1          | 0          | 0          | 100BT Half-duplex, Auto-negotiation enabled  |
    /// | 1          | 0          | 1          | Not used                                     |
    /// | 1          | 1          | 0          | Not used                                     |
    /// | 1          | 1          | 1          | All capable, Auto-negotiation enabled        |
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::PhyCfg;
    ///
    /// let phy_cfg: PhyCfg = PhyCfg::DEFAULT;
    /// assert!(!phy_cfg.opmd());
    /// let phy_cfg: PhyCfg = phy_cfg.software_op();
    /// assert!(phy_cfg.opmd());
    /// let phy_cfg: PhyCfg = phy_cfg.hardware_op();
    /// assert!(!phy_cfg.opmd());
    /// ```
    #[must_use = "hardware_op returns a modified PhyCfg"]
    pub const fn hardware_op(mut self) -> Self {
        self.0 &= !Self::OPMD_MASK;
        self
    }

    /// Enable software configuration of the PHY operation mode.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::PhyCfg;
    ///
    /// let phy_cfg: PhyCfg = PhyCfg::DEFAULT;
    /// assert!(!phy_cfg.opmd());
    /// let phy_cfg: PhyCfg = phy_cfg.software_op();
    /// assert!(phy_cfg.opmd());
    /// ```
    #[must_use = "software_op returns a modified PhyCfg"]
    pub const fn software_op(mut self) -> Self {
        self.0 |= Self::OPMD_MASK;
        self
    }

    /// Get the operation mode.
    ///
    /// This returns an `Err(u8)` with the opmdc bits if the opmdc bits do not
    /// match a valid operation mode.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::{OperationMode, PhyCfg};
    ///
    /// assert_eq!(PhyCfg::DEFAULT.opmdc(), OperationMode::Auto);
    /// ```
    pub const fn opmdc(&self) -> OperationMode {
        // from_raw masks the value
        OperationMode::from_raw(self.0 >> Self::OPMDC_OFFSET)
    }

    /// Set the PHY operation mode.
    ///
    /// Setting this will also call [`PhyCfg::software_op`] to enable the PHY
    /// configuration with the value stored in this register.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::{OperationMode, PhyCfg};
    ///
    /// let phy_cfg: PhyCfg = PhyCfg::DEFAULT;
    /// assert!(!phy_cfg.opmd());
    /// let phy_cfg: PhyCfg = phy_cfg.set_opmdc(OperationMode::PowerDown);
    /// assert!(phy_cfg.opmd());
    /// assert_eq!(phy_cfg.opmdc(), OperationMode::PowerDown);
    /// let phy_cfg: PhyCfg = phy_cfg.set_opmdc(OperationMode::Auto);
    /// assert_eq!(phy_cfg.opmdc(), OperationMode::Auto);
    /// ```
    #[must_use = "set_opmdc returns a modified PhyCfg"]
    pub const fn set_opmdc(mut self, mode: OperationMode) -> Self {
        self = self.software_op();
        self.0 &= !Self::OPMDC_MASK;
        self.0 |= (mode as u8) << Self::OPMDC_OFFSET;
        self
    }

    /// Get the duplex status.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::{DuplexStatus, PhyCfg};
    ///
    /// let phy_cfg: PhyCfg = PhyCfg::DEFAULT;
    /// assert_eq!(phy_cfg.dpx(), DuplexStatus::Half);
    /// ```
    pub const fn dpx(&self) -> DuplexStatus {
        match self.0 & Self::DPX_MASK == Self::DPX_MASK {
            true => DuplexStatus::Full,
            false => DuplexStatus::Half,
        }
    }

    /// Get the speed status.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::{PhyCfg, SpeedStatus};
    ///
    /// let phy_cfg: PhyCfg = PhyCfg::DEFAULT;
    /// assert_eq!(phy_cfg.spd(), SpeedStatus::Mbps10);
    /// ```
    pub const fn spd(&self) -> SpeedStatus {
        match self.0 & Self::SPD_MASK == Self::SPD_MASK {
            true => SpeedStatus::Mbps100,
            false => SpeedStatus::Mbps10,
        }
    }

    /// Get the link status.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::{LinkStatus, PhyCfg};
    ///
    /// let phy_cfg: PhyCfg = PhyCfg::DEFAULT;
    /// assert_eq!(phy_cfg.lnk(), LinkStatus::Down);
    /// ```
    pub const fn lnk(&self) -> LinkStatus {
        match self.0 & Self::LNK_MASK == Self::LNK_MASK {
            true => LinkStatus::Up,
            false => LinkStatus::Down,
        }
    }
}

impl ::core::fmt::Display for PhyCfg {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("PhyCfg")
            .field("opmd", &self.opmdc())
            .field("dpx", &self.dpx())
            .field("spd", &self.spd())
            .field("lnk", &self.lnk())
            .finish()
    }
}

/// Socket Mode Register (Sn_MR).
///
/// This is used by the [`Registers::sn_mr`] and
/// [`Registers::set_sn_mr`] methods.
///
/// [`Registers::set_sn_mr`]: crate::Registers::set_sn_mr
/// [`Registers::sn_mr`]: crate::Registers::sn_mr
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct SocketMode(u8);
impl_boilerplate_for!(SocketMode);

impl SocketMode {
    /// Reset value of the socket mode register.
    pub const RESET: u8 = 0x00;

    /// Default value.
    ///
    /// This is the same as `default`, but as a `const` value.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::SocketMode;
    ///
    /// assert_eq!(SocketMode::DEFAULT, SocketMode::default());
    /// ```
    pub const DEFAULT: Self = Self(Self::RESET);

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
    /// This returns an `Err(u8)` with the protocol bits if the protocol bits
    /// do not match a valid protocol.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::{Protocol, SocketMode};
    ///
    /// let sn_mr: SocketMode = SocketMode::DEFAULT;
    /// assert_eq!(sn_mr.protocol(), Ok(Protocol::Closed));
    /// ```
    pub const fn protocol(&self) -> Result<Protocol, u8> {
        Protocol::from_raw(self.0 & Self::PROTOCOL_MASK)
    }

    /// Set the protocol.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::{Protocol, SocketMode};
    ///
    /// const SN_MR: SocketMode = SocketMode::DEFAULT.set_protocol(Protocol::Tcp);
    /// assert_eq!(SN_MR.protocol(), Ok(Protocol::Tcp));
    /// ```
    pub const fn set_protocol(mut self, protocol: Protocol) -> Self {
        self.0 = (self.0 & 0xF0) | ((protocol as u8) & 0xF);
        self
    }

    /// Multicasting.
    ///
    /// This applies only for a socket with the UDP protocol.
    ///
    /// To use multicasting [`Registers::sn_dipr`] and [`Registers::sn_dport`]
    /// should be configured with the multicast group IP and port number
    /// before the socket is opened.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::SocketMode;
    ///
    /// let sn_mr: SocketMode = SocketMode::DEFAULT;
    /// assert!(!sn_mr.multi_enabled());
    /// let sn_mr: SocketMode = sn_mr.enable_multi();
    /// assert!(sn_mr.multi_enabled());
    /// let sn_mr: SocketMode = sn_mr.disable_multi();
    /// assert!(!sn_mr.multi_enabled());
    /// ```
    ///
    /// [`Registers::sn_dipr`]: crate::Registers::sn_dipr
    /// [`Registers::sn_dport`]: crate::Registers::sn_dport
    pub const fn multi_enabled(&self) -> bool {
        self.0 & Self::MULTI_MASK != 0
    }

    /// Enable multicasting.
    #[must_use = "enable_multi returns a modified SocketMode"]
    pub const fn enable_multi(mut self) -> Self {
        self.0 |= Self::MULTI_MASK;
        self
    }

    /// Disable multicasting.
    #[must_use = "disable_multi returns a modified SocketMode"]
    pub const fn disable_multi(mut self) -> Self {
        self.0 &= !Self::MULTI_MASK;
        self
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
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::SocketMode;
    ///
    /// let sn_mr: SocketMode = SocketMode::DEFAULT;
    /// assert!(!sn_mr.mfen_enabled());
    /// let sn_mr: SocketMode = sn_mr.enable_mfen();
    /// assert!(sn_mr.mfen_enabled());
    /// let sn_mr: SocketMode = sn_mr.disable_mfen();
    /// assert!(!sn_mr.mfen_enabled());
    /// ```
    pub const fn mfen_enabled(&self) -> bool {
        self.0 & Self::MFEN_MASK != 0
    }

    /// Enable MAC filter.
    #[must_use = "enable_mfen returns a modified SocketMode"]
    pub const fn enable_mfen(mut self) -> Self {
        self.0 |= Self::MFEN_MASK;
        self
    }

    /// Disable MAC filter.
    #[must_use = "disable_mfen returns a modified SocketMode"]
    pub const fn disable_mfen(mut self) -> Self {
        self.0 &= !Self::MFEN_MASK;
        self
    }

    /// Broadcast blocking.
    ///
    /// This applies only for a socket with the MACRAW or UDP protocol.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::SocketMode;
    ///
    /// let sn_mr: SocketMode = SocketMode::DEFAULT;
    /// assert!(!sn_mr.bcastb_enabled());
    /// let sn_mr: SocketMode = sn_mr.enable_bcastb();
    /// assert!(sn_mr.bcastb_enabled());
    /// let sn_mr: SocketMode = sn_mr.disable_bcastb();
    /// assert!(!sn_mr.bcastb_enabled());
    /// ```
    pub const fn bcastb_enabled(&self) -> bool {
        self.0 & Self::BCASTB_MASK != 0
    }

    /// Enable broadcast blocking.
    #[must_use = "enable_bcastb returns a modified SocketMode"]
    pub const fn enable_bcastb(mut self) -> Self {
        self.0 |= Self::BCASTB_MASK;
        self
    }

    /// Disable broadcast blocking.
    #[must_use = "disable_bcastb returns a modified SocketMode"]
    pub const fn disable_bcastb(mut self) -> Self {
        self.0 &= !Self::BCASTB_MASK;
        self
    }

    /// Use no delayed ACK.
    ///
    /// This applies only for a socket with the TCP protocol.
    ///
    /// When enabled the ACK packet is sent without delay as soon as a data
    /// packet is received from a peer.
    /// When disabled the ACK packet is sent after waiting for the time
    /// configured by [`rtr`].
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::SocketMode;
    ///
    /// let sn_mr: SocketMode = SocketMode::DEFAULT;
    /// assert!(!sn_mr.nd_enabled());
    /// let sn_mr: SocketMode = sn_mr.enable_nd();
    /// assert!(sn_mr.nd_enabled());
    /// let sn_mr: SocketMode = sn_mr.disable_nd();
    /// assert!(!sn_mr.nd_enabled());
    /// ```
    ///
    /// [`rtr`]: crate::Registers::rtr
    pub const fn nd_enabled(&self) -> bool {
        self.0 & Self::ND_MASK != 0
    }

    /// Disable no delayed ACK.
    #[must_use = "disable_nd returns a modified SocketMode"]
    pub const fn disable_nd(mut self) -> Self {
        self.0 &= !Self::ND_MASK;
        self
    }

    /// Enable no delayed ACK.
    #[must_use = "enable_nd returns a modified SocketMode"]
    pub const fn enable_nd(mut self) -> Self {
        self.0 |= Self::ND_MASK;
        self
    }

    /// Multicast IGMP version.
    ///
    /// This applies only for a socket with the UDP protocol.
    ///
    /// This field configures the version for IGMP messages (join/leave/report).
    ///
    /// * `false` IGMP version 2
    /// * `true` IGMP version 1
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::SocketMode;
    ///
    /// let sn_mr: SocketMode = SocketMode::DEFAULT;
    /// assert!(!sn_mr.mc());
    /// let sn_mr: SocketMode = sn_mr.set_igmp_v1();
    /// assert!(sn_mr.mc());
    /// let sn_mr: SocketMode = sn_mr.set_igmp_v2();
    /// assert!(!sn_mr.mc());
    /// ```
    pub const fn mc(&self) -> bool {
        self.0 & Self::MC_MASK != 0
    }

    /// Set IGMP version 1.
    #[must_use = "set_igmp_v1 returns a modified SocketMode"]
    pub const fn set_igmp_v1(mut self) -> Self {
        self.0 |= Self::MC_MASK;
        self
    }

    /// Set IGMP version 2.
    #[must_use = "set_igmp_v2 returns a modified SocketMode"]
    pub const fn set_igmp_v2(mut self) -> Self {
        self.0 &= !Self::MC_MASK;
        self
    }

    /// Multicast blocking.
    ///
    /// This applies only for a socket with the [MACRAW] protocol.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::SocketMode;
    ///
    /// let sn_mr: SocketMode = SocketMode::DEFAULT;
    /// assert!(!sn_mr.mmb_enabled());
    /// let sn_mr: SocketMode = sn_mr.enable_mmb();
    /// assert!(sn_mr.mmb_enabled());
    /// let sn_mr: SocketMode = sn_mr.disable_mmb();
    /// assert!(!sn_mr.mmb_enabled());
    /// ```
    ///
    /// [MACRAW]: crate::Protocol::Macraw
    pub const fn mmb_enabled(&self) -> bool {
        self.0 & Self::MMB_MASK != 0
    }

    /// Enable multicast blocking.
    #[must_use = "enable_mmb returns a modified SocketMode"]
    pub const fn enable_mmb(mut self) -> Self {
        self.0 |= Self::MMB_MASK;
        self
    }

    /// Disable multicast blocking.
    #[must_use = "disable_mmb returns a modified SocketMode"]
    pub const fn disable_mmb(mut self) -> Self {
        self.0 &= !Self::MMB_MASK;
        self
    }

    /// Unicast blocking enabled.
    ///
    /// This applies only for a socket with the UDP protocol.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::SocketMode;
    ///
    /// let sn_mr: SocketMode = SocketMode::DEFAULT;
    /// assert!(!sn_mr.ucastb_enabled());
    /// let sn_mr: SocketMode = sn_mr.enable_ucastb();
    /// assert!(sn_mr.ucastb_enabled());
    /// let sn_mr: SocketMode = sn_mr.disable_ucastb();
    /// assert!(!sn_mr.ucastb_enabled());
    /// ```
    pub const fn ucastb_enabled(&self) -> bool {
        self.0 & Self::UCASTB_MASK != 0
    }

    /// Enable unicast blocking.
    #[must_use = "enable_ucastb returns a modified SocketMode"]
    pub const fn enable_ucastb(mut self) -> Self {
        self.0 |= Self::UCASTB_MASK;
        self
    }

    /// Disable unicast blocking.
    #[must_use = "disable_ucastb returns a modified SocketMode"]
    pub const fn disable_ucastb(mut self) -> Self {
        self.0 &= !Self::UCASTB_MASK;
        self
    }

    /// IPV6 packet blocking.
    ///
    /// This applies only for a socket with the [MACRAW] protocol.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::SocketMode;
    ///
    /// let sn_mr: SocketMode = SocketMode::DEFAULT;
    /// assert!(!sn_mr.mip6b_enabled());
    /// let sn_mr: SocketMode = sn_mr.enable_mip6b();
    /// assert!(sn_mr.mip6b_enabled());
    /// let sn_mr: SocketMode = sn_mr.disable_mip6b();
    /// assert!(!sn_mr.mip6b_enabled());
    /// ```
    ///
    /// [MACRAW]: crate::Protocol::Macraw
    pub const fn mip6b_enabled(&self) -> bool {
        self.0 & Self::MIP6B_MASK != 0
    }

    /// Enable IPV6 packet blocking.
    #[must_use = "enable_mip6b returns a modified SocketMode"]
    pub const fn enable_mip6b(mut self) -> Self {
        self.0 |= Self::MIP6B_MASK;
        self
    }

    /// Disable IPV6 packet blocking.
    #[must_use = "disable_mip6b returns a modified SocketMode"]
    pub const fn disable_mip6b(mut self) -> Self {
        self.0 &= !Self::MIP6B_MASK;
        self
    }
}

impl ::core::fmt::Display for SocketMode {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("SocketMode")
            .field("protocol", &self.protocol())
            .field("multi_enabled", &self.multi_enabled())
            .field("mfen_enabled", &self.mfen_enabled())
            .field("bcastb_enabled", &self.bcastb_enabled())
            .field("nd_enabled", &self.nd_enabled())
            .field("mc", &self.mc())
            .field("mmb_enabled", &self.mmb_enabled())
            .field("ucastb_enabled", &self.ucastb_enabled())
            .field("mip6b_enabled", &self.mip6b_enabled())
            .finish()
    }
}

/// Socket Interrupt Register (Sn_IR).
///
/// Indicated the socket status, such as connection, termination,
/// receiving data, and timeout.
///
/// This is used by the [`Registers::sn_ir`] and
/// [`Registers::set_sn_ir`] methods.
///
/// [`Registers::sn_ir`]: crate::Registers::sn_ir
/// [`Registers::set_sn_ir`]: crate::Registers::set_sn_ir
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct SocketInterrupt(u8);
impl_boilerplate_for!(SocketInterrupt);

impl SocketInterrupt {
    /// Socket interrupt status register (Sn_IR) reset value.
    pub const RESET: u8 = 0x00;

    /// Default value.
    ///
    /// This is the same as `default`, but as a `const` value.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::SocketInterrupt;
    ///
    /// assert_eq!(SocketInterrupt::DEFAULT, SocketInterrupt::default());
    /// ```
    pub const DEFAULT: Self = Self(Self::RESET);

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
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::SocketInterrupt;
    ///
    /// let sir: SocketInterrupt = SocketInterrupt::DEFAULT;
    /// assert!(!sir.con_raised());
    /// # assert!(sir.clear_con().con_raised());
    /// ```
    pub const fn con_raised(&self) -> bool {
        self.0 & Self::CON_MASK != 0
    }

    /// Clear the `CON` interrupt by writing `1`.
    #[must_use = "clear_con returns a modified SocketInterrupt"]
    pub const fn clear_con(mut self) -> Self {
        self.0 |= Self::CON_MASK;
        self
    }

    /// Get the value of the `DISCON` interrupt.
    ///
    /// This is issued when FIN or FIN/ACK packet is received from a peer.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::SocketInterrupt;
    ///
    /// let sir: SocketInterrupt = SocketInterrupt::DEFAULT;
    /// assert!(!sir.discon_raised());
    /// # assert!(sir.clear_discon().discon_raised());
    /// ```
    pub const fn discon_raised(&self) -> bool {
        self.0 & Self::DISCON_MASK != 0
    }

    /// Clear the `DISCON` interrupt by writing `1`.
    #[must_use = "clear_discon returns a modified SocketInterrupt"]
    pub const fn clear_discon(mut self) -> Self {
        self.0 |= Self::DISCON_MASK;
        self
    }

    /// Get the value of the `RECV` interrupt.
    ///
    /// This is issued whenever data is received from a peer.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::SocketInterrupt;
    ///
    /// let sir: SocketInterrupt = SocketInterrupt::DEFAULT;
    /// assert!(!sir.recv_raised());
    /// # assert!(sir.clear_recv().recv_raised());
    /// ```
    pub const fn recv_raised(&self) -> bool {
        self.0 & Self::RECV_MASK != 0
    }

    /// Clear the `RECV` interrupt by writing `1`.
    #[must_use = "clear_recv returns a modified SocketInterrupt"]
    pub const fn clear_recv(mut self) -> Self {
        self.0 |= Self::RECV_MASK;
        self
    }

    /// Get the value of the `TIMEOUT` interrupt.
    ///
    /// This is issued when ARP<sub>TO</sub> or TCP<sub>TO</sub> occurs.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::SocketInterrupt;
    ///
    /// let sir: SocketInterrupt = SocketInterrupt::DEFAULT;
    /// assert!(!sir.timeout_raised());
    /// # assert!(sir.clear_timeout().timeout_raised());
    /// ```
    pub const fn timeout_raised(&self) -> bool {
        self.0 & Self::TIMEOUT_MASK != 0
    }

    /// Clear the `TIMEOUT` interrupt by writing `1`.
    #[must_use = "clear_timeout returns a modified SocketInterrupt"]
    pub const fn clear_timeout(mut self) -> Self {
        self.0 |= Self::TIMEOUT_MASK;
        self
    }

    /// Get the value of the `SENDOK` interrupt.
    ///
    /// This is issued when [SEND] command is completed.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::SocketInterrupt;
    ///
    /// let sir: SocketInterrupt = SocketInterrupt::DEFAULT;
    /// assert!(!sir.sendok_raised());
    /// # assert!(sir.clear_sendok().sendok_raised());
    /// ```
    ///
    /// [SEND]: crate::SocketCommand::Send
    pub const fn sendok_raised(&self) -> bool {
        self.0 & Self::SENDOK_MASK != 0
    }

    /// Clear the `SENDOK` interrupt by writing `1`.
    #[must_use = "clear_sendok returns a modified SocketInterrupt"]
    pub const fn clear_sendok(mut self) -> Self {
        self.0 |= Self::SENDOK_MASK;
        self
    }
}

impl ::core::fmt::Display for SocketInterrupt {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("SocketInterrupt")
            .field("con_raised", &self.con_raised())
            .field("discon_raised", &self.discon_raised())
            .field("recv_raised", &self.recv_raised())
            .field("timeout_raised", &self.timeout_raised())
            .field("sendok_raised", &self.sendok_raised())
            .finish()
    }
}

/// Socket Interrupt Mask Register (Sn_IMR).
///
/// This is used by the [`Registers::sn_imr`] and
/// [`Registers::set_sn_imr`] methods.
///
/// See the [`SocketInterrupt`] structure for more information about the
/// individual interrupts.
///
/// [`Registers::sn_imr`]: crate::Registers::sn_imr
/// [`Registers::set_sn_imr`]: crate::Registers::set_sn_imr
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct SocketInterruptMask(u8);
impl_boilerplate_for!(SocketInterruptMask);

impl SocketInterruptMask {
    /// Socket interrupt mask register (Sn_IMR) reset value.
    pub const RESET: u8 = 0xFF;

    /// Default value.
    ///
    /// This is the same as `default`, but as a `const` value.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::SocketInterruptMask;
    ///
    /// assert_eq!(SocketInterruptMask::DEFAULT, SocketInterruptMask::default());
    /// ```
    pub const DEFAULT: Self = Self(Self::RESET);

    /// Mask all socket interrupts.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::SocketInterruptMask;
    ///
    /// assert!(SocketInterruptMask::ALL_MASKED.con_masked());
    /// assert!(SocketInterruptMask::ALL_MASKED.discon_masked());
    /// assert!(SocketInterruptMask::ALL_MASKED.recv_masked());
    /// assert!(SocketInterruptMask::ALL_MASKED.timeout_masked());
    /// assert!(SocketInterruptMask::ALL_MASKED.sendok_masked());
    /// ```
    pub const ALL_MASKED: SocketInterruptMask = SocketInterruptMask(0xE0);

    /// Check if the `CON` interrupt is masked.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::SocketInterruptMask;
    ///
    /// let simr: SocketInterruptMask = SocketInterruptMask::DEFAULT;
    /// assert!(!simr.con_masked());
    /// let simr: SocketInterruptMask = simr.mask_con();
    /// assert!(simr.con_masked());
    /// let simr: SocketInterruptMask = simr.unmask_con();
    /// assert!(!simr.con_masked());
    /// ```
    pub const fn con_masked(&self) -> bool {
        self.0 & SocketInterrupt::CON_MASK == 0
    }

    /// Unmask the `CON` interrupt.
    #[must_use = "unmask_con returns a modified SocketInterruptMask"]
    pub const fn unmask_con(mut self) -> Self {
        self.0 |= SocketInterrupt::CON_MASK;
        self
    }

    /// Mask the `CON` interrupt.
    #[must_use = "mask_con returns a modified SocketInterruptMask"]
    pub const fn mask_con(mut self) -> Self {
        self.0 &= !SocketInterrupt::CON_MASK;
        self
    }

    /// Check if the `DISCON` interrupt is masked.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::SocketInterruptMask;
    ///
    /// let simr: SocketInterruptMask = SocketInterruptMask::DEFAULT;
    /// assert!(!simr.discon_masked());
    /// let simr: SocketInterruptMask = simr.mask_discon();
    /// assert!(simr.discon_masked());
    /// let simr: SocketInterruptMask = simr.unmask_discon();
    /// assert!(!simr.discon_masked());
    /// ```
    pub const fn discon_masked(&self) -> bool {
        self.0 & SocketInterrupt::DISCON_MASK == 0
    }

    /// Unmask the `DISCON` interrupt.
    #[must_use = "unmask_discon returns a modified SocketInterruptMask"]
    pub const fn unmask_discon(mut self) -> Self {
        self.0 |= SocketInterrupt::DISCON_MASK;
        self
    }

    /// Mask the `DISCON` interrupt.
    #[must_use = "mask_discon returns a modified SocketInterruptMask"]
    pub const fn mask_discon(mut self) -> Self {
        self.0 &= !SocketInterrupt::DISCON_MASK;
        self
    }

    /// Check if the `RECV` interrupt is masked.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::SocketInterruptMask;
    ///
    /// let simr: SocketInterruptMask = SocketInterruptMask::DEFAULT;
    /// assert!(!simr.recv_masked());
    /// let simr: SocketInterruptMask = simr.mask_recv();
    /// assert!(simr.recv_masked());
    /// let simr: SocketInterruptMask = simr.unmask_recv();
    /// assert!(!simr.recv_masked());
    /// ```
    pub const fn recv_masked(&self) -> bool {
        self.0 & SocketInterrupt::RECV_MASK == 0
    }

    /// Unmask the `RECV` interrupt.
    #[must_use = "unmask_recv returns a modified SocketInterruptMask"]
    pub const fn unmask_recv(mut self) -> Self {
        self.0 |= SocketInterrupt::RECV_MASK;
        self
    }

    /// Mask the `RECV` interrupt.
    #[must_use = "mask_recv returns a modified SocketInterruptMask"]
    pub const fn mask_recv(mut self) -> Self {
        self.0 &= !SocketInterrupt::RECV_MASK;
        self
    }

    /// Check if the `TIMEOUT` interrupt is masked.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::SocketInterruptMask;
    ///
    /// let simr: SocketInterruptMask = SocketInterruptMask::DEFAULT;
    /// assert!(!simr.timeout_masked());
    /// let simr: SocketInterruptMask = simr.mask_timeout();
    /// assert!(simr.timeout_masked());
    /// let simr: SocketInterruptMask = simr.unmask_timeout();
    /// assert!(!simr.timeout_masked());
    /// ```
    pub const fn timeout_masked(&self) -> bool {
        self.0 & SocketInterrupt::TIMEOUT_MASK == 0
    }

    /// Unmask the `TIMEOUT` interrupt.
    #[must_use = "unmask_timeout returns a modified SocketInterruptMask"]
    pub const fn unmask_timeout(mut self) -> Self {
        self.0 |= SocketInterrupt::TIMEOUT_MASK;
        self
    }

    /// Mask the `TIMEOUT` interrupt.
    #[must_use = "mask_timeout returns a modified SocketInterruptMask"]
    pub const fn mask_timeout(mut self) -> Self {
        self.0 &= !SocketInterrupt::TIMEOUT_MASK;
        self
    }

    /// Check if the `SENDOK` interrupt is masked.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::SocketInterruptMask;
    ///
    /// let simr: SocketInterruptMask = SocketInterruptMask::DEFAULT;
    /// assert!(!simr.sendok_masked());
    /// let simr: SocketInterruptMask = simr.mask_sendok();
    /// assert!(simr.sendok_masked());
    /// let simr: SocketInterruptMask = simr.unmask_sendok();
    /// assert!(!simr.sendok_masked());
    /// ```
    pub const fn sendok_masked(&self) -> bool {
        self.0 & SocketInterrupt::SENDOK_MASK == 0
    }

    /// Unmask the `SENDOK` interrupt.
    #[must_use = "unmask_sendok returns a modified SocketInterruptMask"]
    pub const fn unmask_sendok(mut self) -> Self {
        self.0 |= SocketInterrupt::SENDOK_MASK;
        self
    }

    /// Mask the `SENDOK` interrupt.
    #[must_use = "mask_sendok returns a modified SocketInterruptMask"]
    pub const fn mask_sendok(mut self) -> Self {
        self.0 &= !SocketInterrupt::SENDOK_MASK;
        self
    }
}

impl ::core::fmt::Display for SocketInterruptMask {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("SocketInterruptMask")
            .field("con_masked", &self.con_masked())
            .field("discon_masked", &self.discon_masked())
            .field("recv_masked", &self.recv_masked())
            .field("timeout_masked", &self.timeout_masked())
            .field("sendok_masked", &self.sendok_masked())
            .finish()
    }
}
