/// W5500 common register addresses.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u16)]
#[allow(non_camel_case_types)]
pub enum Reg {
    /// Address of the MR register.
    MR = 0x0000,
    /// Address of the GAR register.
    GAR = 0x0001,
    /// Address of the SUBR register.
    SUBR = 0x0005,
    /// Address of the SHAR register.
    SHAR = 0x0009,
    /// Address of the SIPR register.
    SIPR = 0x000F,
    /// Address of the INTLEVEL register.
    INTLEVEL = 0x0013,
    /// Address of the IR register.
    IR = 0x0015,
    /// Address of the IMR register.
    IMR = 0x0016,
    /// Address of the SIR register.
    SIR = 0x0017,
    /// Address of the SIMR register.
    SIMR = 0x0018,
    /// Address of the RTR register.
    RTR = 0x0019,
    /// Address of the RCR register.
    RCR = 0x001B,
    /// Address of the PTIMER register.
    PTIMER = 0x001C,
    /// Address of the PMAGIC register.
    PMAGIC = 0x001D,
    /// Address of the PHAR register.
    PHAR = 0x001E,
    /// Address of the PSID register.
    PSID = 0x0024,
    /// Address of the PMRU register.
    PMRU = 0x0026,
    /// Address of the UIPR register.
    UIPR = 0x0028,
    /// Address of the UPORTR register.
    UPORTR = 0x002C,
    /// Address of the PHYCFGR register.
    PHYCFGR = 0x002E,
    /// Address of the VERSIONR register.
    VERSIONR = 0x0039,
}

impl From<Reg> for u16 {
    fn from(reg: Reg) -> Self {
        reg as u16
    }
}

impl TryFrom<u16> for Reg {
    type Error = u16;

    fn try_from(val: u16) -> Result<Self, Self::Error> {
        match val {
            x if x == Self::MR as u16 => Ok(Self::MR),
            x if x == Self::GAR as u16 => Ok(Self::GAR),
            x if x == Self::SUBR as u16 => Ok(Self::SUBR),
            x if x == Self::SHAR as u16 => Ok(Self::SHAR),
            x if x == Self::SIPR as u16 => Ok(Self::SIPR),
            x if x == Self::INTLEVEL as u16 => Ok(Self::INTLEVEL),
            x if x == Self::IR as u16 => Ok(Self::IR),
            x if x == Self::IMR as u16 => Ok(Self::IMR),
            x if x == Self::SIR as u16 => Ok(Self::SIR),
            x if x == Self::SIMR as u16 => Ok(Self::SIMR),
            x if x == Self::RTR as u16 => Ok(Self::RTR),
            x if x == Self::RCR as u16 => Ok(Self::RCR),
            x if x == Self::PTIMER as u16 => Ok(Self::PTIMER),
            x if x == Self::PMAGIC as u16 => Ok(Self::PMAGIC),
            x if x == Self::PHAR as u16 => Ok(Self::PHAR),
            x if x == Self::PSID as u16 => Ok(Self::PSID),
            x if x == Self::PMRU as u16 => Ok(Self::PMRU),
            x if x == Self::UIPR as u16 => Ok(Self::UIPR),
            x if x == Self::UPORTR as u16 => Ok(Self::UPORTR),
            x if x == Self::PHYCFGR as u16 => Ok(Self::PHYCFGR),
            x if x == Self::VERSIONR as u16 => Ok(Self::VERSIONR),
            _ => Err(val),
        }
    }
}

impl Reg {
    /// Get the address of the register.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::Reg;
    ///
    /// assert_eq!(Reg::VERSIONR.addr(), 0x0039);
    /// ```
    pub const fn addr(self) -> u16 {
        self as u16
    }

    /// Get the register width in bytes.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::Reg;
    ///
    /// // mode is 1 byte
    /// assert_eq!(Reg::MR.width(), 1);
    ///
    /// // port is 2 bytes
    /// assert_eq!(Reg::UPORTR.width(), 2);
    ///
    /// // IPv4 is 4 bytes
    /// assert_eq!(Reg::SIPR.width(), 4);
    ///
    /// // EUI-48 MAC is 6 bytes
    /// assert_eq!(Reg::SHAR.width(), 6);
    /// ```
    pub const fn width(self) -> u8 {
        match self {
            Reg::MR => 1,
            Reg::GAR => 4,
            Reg::SUBR => 4,
            Reg::SHAR => 6,
            Reg::SIPR => 4,
            Reg::INTLEVEL => 2,
            Reg::IR => 1,
            Reg::IMR => 1,
            Reg::SIR => 1,
            Reg::SIMR => 1,
            Reg::RTR => 2,
            Reg::RCR => 1,
            Reg::PTIMER => 1,
            Reg::PMAGIC => 1,
            Reg::PHAR => 6,
            Reg::PSID => 2,
            Reg::PMRU => 2,
            Reg::UIPR => 4,
            Reg::UPORTR => 2,
            Reg::PHYCFGR => 1,
            Reg::VERSIONR => 1,
        }
    }

    /// Returns `true` if the register is read-only.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::Reg;
    ///
    /// assert!(Reg::VERSIONR.is_ro());
    /// assert!(!Reg::MR.is_ro());
    /// ```
    pub const fn is_ro(self) -> bool {
        matches!(self, Reg::UIPR | Reg::UPORTR | Reg::VERSIONR)
    }
}

/// W5500 socket register addresses.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u16)]
#[allow(non_camel_case_types)]
pub enum SnReg {
    /// Address of the SN_MR register.
    MR = 0x0000,
    /// Address of the SN_CR register.
    CR = 0x0001,
    /// Address of the SN_IR register.
    IR = 0x0002,
    /// Address of the SN_SR register.
    SR = 0x0003,
    /// Address of the SN_PORT register.
    PORT = 0x0004,
    /// Address of the SN_DHAR register.
    DHAR = 0x0006,
    /// Address of the SN_DIPR register.
    DIPR = 0x000C,
    /// Address of the SN_DPORT register.
    DPORT = 0x0010,
    /// Address of the SN_MSSR register.
    MSSR = 0x0012,
    /// Address of the SN_TOS register.
    TOS = 0x0015,
    /// Address of the SN_TTL register.
    TTL = 0x0016,
    /// Address of the SN_RXBUF_SIZE register.
    RXBUF_SIZE = 0x001E,
    /// Address of the SN_TXBUF_SIZE register.
    TXBUF_SIZE = 0x001F,
    /// Address of the SN_TX_FSR register.
    TX_FSR = 0x0020,
    /// Address of the SN_TX_RD register.
    TX_RD = 0x0022,
    /// Address of the SN_TX_WR register.
    TX_WR = 0x0024,
    /// Address of the SN_RX_RSR register.
    RX_RSR = 0x0026,
    /// Address of the SN_RX_RD register.
    RX_RD = 0x0028,
    /// Address of the SN_RX_WR register.
    RX_WR = 0x002A,
    /// Address of the SN_IMR register.
    IMR = 0x002C,
    /// Address of the SN_FRAG register.
    FRAG = 0x002D,
    /// Address of the SN_KPALVTR register.
    KPALVTR = 0x002F,
}

impl From<SnReg> for u16 {
    fn from(snreg: SnReg) -> Self {
        snreg as u16
    }
}

impl TryFrom<u16> for SnReg {
    type Error = u16;

    fn try_from(val: u16) -> Result<Self, Self::Error> {
        match val {
            x if x == Self::MR as u16 => Ok(Self::MR),
            x if x == Self::CR as u16 => Ok(Self::CR),
            x if x == Self::IR as u16 => Ok(Self::IR),
            x if x == Self::SR as u16 => Ok(Self::SR),
            x if x == Self::PORT as u16 => Ok(Self::PORT),
            x if x == Self::DHAR as u16 => Ok(Self::DHAR),
            x if x == Self::DIPR as u16 => Ok(Self::DIPR),
            x if x == Self::DPORT as u16 => Ok(Self::DPORT),
            x if x == Self::MSSR as u16 => Ok(Self::MSSR),
            x if x == Self::TOS as u16 => Ok(Self::TOS),
            x if x == Self::TTL as u16 => Ok(Self::TTL),
            x if x == Self::RXBUF_SIZE as u16 => Ok(Self::RXBUF_SIZE),
            x if x == Self::TXBUF_SIZE as u16 => Ok(Self::TXBUF_SIZE),
            x if x == Self::TX_FSR as u16 => Ok(Self::TX_FSR),
            x if x == Self::TX_RD as u16 => Ok(Self::TX_RD),
            x if x == Self::TX_WR as u16 => Ok(Self::TX_WR),
            x if x == Self::RX_RSR as u16 => Ok(Self::RX_RSR),
            x if x == Self::RX_RD as u16 => Ok(Self::RX_RD),
            x if x == Self::RX_WR as u16 => Ok(Self::RX_WR),
            x if x == Self::IMR as u16 => Ok(Self::IMR),
            x if x == Self::FRAG as u16 => Ok(Self::FRAG),
            x if x == Self::KPALVTR as u16 => Ok(Self::KPALVTR),
            _ => Err(val),
        }
    }
}

impl SnReg {
    /// Get the address of the socket register.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::SnReg;
    ///
    /// assert_eq!(SnReg::PORT.addr(), 0x0004);
    /// ```
    pub const fn addr(self) -> u16 {
        self as u16
    }

    /// Get the register width in bytes.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::SnReg;
    ///
    /// // mode is 1 byte
    /// assert_eq!(SnReg::MR.width(), 1);
    ///
    /// // port is 2 bytes
    /// assert_eq!(SnReg::PORT.width(), 2);
    ///
    /// // IPv4 is 4 bytes
    /// assert_eq!(SnReg::DIPR.width(), 4);
    ///
    /// // EUI-48 MAC is 6 bytes
    /// assert_eq!(SnReg::DHAR.width(), 6);
    /// ```
    pub const fn width(self) -> u8 {
        match self {
            SnReg::MR => 1,
            SnReg::CR => 1,
            SnReg::IR => 1,
            SnReg::SR => 1,
            SnReg::PORT => 2,
            SnReg::DHAR => 6,
            SnReg::DIPR => 4,
            SnReg::DPORT => 2,
            SnReg::MSSR => 2,
            SnReg::TOS => 1,
            SnReg::TTL => 1,
            SnReg::RXBUF_SIZE => 1,
            SnReg::TXBUF_SIZE => 1,
            SnReg::TX_FSR => 2,
            SnReg::TX_RD => 2,
            SnReg::TX_WR => 2,
            SnReg::RX_RSR => 2,
            SnReg::RX_RD => 2,
            SnReg::RX_WR => 2,
            SnReg::IMR => 1,
            SnReg::FRAG => 2,
            SnReg::KPALVTR => 1,
        }
    }

    /// Returns `true` if the register is read-only.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::SnReg;
    ///
    /// assert!(SnReg::RX_RSR.is_ro());
    /// assert!(!SnReg::MR.is_ro());
    /// ```
    pub const fn is_ro(self) -> bool {
        matches!(self, Self::SR | Self::TX_FSR | Self::TX_RD | Self::RX_RSR)
    }
}
