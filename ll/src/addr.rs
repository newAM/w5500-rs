/// W5500 common register addresses.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u16)]
#[allow(non_camel_case_types)]
pub enum Reg {
    /// Address of the MR register.
    MR = 0x0000,
    /// Address of the GAR register, index 0.
    GAR0 = 0x0001,
    /// Address of the GAR register, index 1.
    GAR1 = 0x0002,
    /// Address of the GAR register, index 2.
    GAR2 = 0x0003,
    /// Address of the GAR register, index 3.
    GAR3 = 0x0004,
    /// Address of the SUBR register, index 0.
    SUBR0 = 0x0005,
    /// Address of the SUBR register, index 1.
    SUBR1 = 0x0006,
    /// Address of the SUBR register, index 2.
    SUBR2 = 0x0007,
    /// Address of the SUBR register, index 3.
    SUBR3 = 0x0008,
    /// Address of the SHAR register, index 0.
    SHAR0 = 0x0009,
    /// Address of the SHAR register, index 1.
    SHAR1 = 0x000A,
    /// Address of the SHAR register, index 2.
    SHAR2 = 0x000B,
    /// Address of the SHAR register, index 3.
    SHAR3 = 0x000C,
    /// Address of the SHAR register, index 4.
    SHAR4 = 0x000D,
    /// Address of the SHAR register, index 5.
    SHAR5 = 0x000E,
    /// Address of the SIPR register, index 0.
    SIPR0 = 0x000F,
    /// Address of the SIPR register, index 1.
    SIPR1 = 0x0010,
    /// Address of the SIPR register, index 2.
    SIPR2 = 0x0011,
    /// Address of the SIPR register, index 3.
    SIPR3 = 0x0012,
    /// Address of the INTLEVEL register, index 0.
    INTLEVEL0 = 0x0013,
    /// Address of the INTLEVEL register, index 1.
    INTLEVEL1 = 0x0014,
    /// Address of the IR register.
    IR = 0x0015,
    /// Address of the IMR register.
    IMR = 0x0016,
    /// Address of the SIR register.
    SIR = 0x0017,
    /// Address of the SIMR register.
    SIMR = 0x0018,
    /// Address of the RTR register, index 0.
    RTR0 = 0x0019,
    /// Address of the RTR register, index 1.
    RTR1 = 0x001A,
    /// Address of the RCR register.
    RCR = 0x001B,
    /// Address of the PTIMER register.
    PTIMER = 0x001C,
    /// Address of the PMAGIC register.
    PMAGIC = 0x001D,
    /// Address of the PHAR register, index 0.
    PHAR0 = 0x001E,
    /// Address of the PHAR register, index 1.
    PHAR1 = 0x001F,
    /// Address of the PHAR register, index 2.
    PHAR2 = 0x0020,
    /// Address of the PHAR register, index 3.
    PHAR3 = 0x0021,
    /// Address of the PHAR register, index 4.
    PHAR4 = 0x0022,
    /// Address of the PHAR register, index 5.
    PHAR5 = 0x0023,
    /// Address of the PSID register, index 0.
    PSID0 = 0x0024,
    /// Address of the PSID register, index 1.
    PSID1 = 0x0025,
    /// Address of the PMRU register, index 0.
    PMRU0 = 0x0026,
    /// Address of the PMRU register, index 1.
    PMRU1 = 0x0027,
    /// Address of the UIPR register, index 0.
    UIPR0 = 0x0028,
    /// Address of the UIPR register, index 1.
    UIPR1 = 0x0029,
    /// Address of the UIPR register, index 2.
    UIPR2 = 0x002A,
    /// Address of the UIPR register, index 3.
    UIPR3 = 0x002B,
    /// Address of the UPORTR register, index 0.
    UPORTR0 = 0x002C,
    /// Address of the UPORTR register, index 1.
    UPORTR1 = 0x002D,
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
            x if x == Self::GAR0 as u16 => Ok(Self::GAR0),
            x if x == Self::GAR1 as u16 => Ok(Self::GAR1),
            x if x == Self::GAR2 as u16 => Ok(Self::GAR2),
            x if x == Self::GAR3 as u16 => Ok(Self::GAR3),
            x if x == Self::SUBR0 as u16 => Ok(Self::SUBR0),
            x if x == Self::SUBR1 as u16 => Ok(Self::SUBR1),
            x if x == Self::SUBR2 as u16 => Ok(Self::SUBR2),
            x if x == Self::SUBR3 as u16 => Ok(Self::SUBR3),
            x if x == Self::SHAR0 as u16 => Ok(Self::SHAR0),
            x if x == Self::SHAR1 as u16 => Ok(Self::SHAR1),
            x if x == Self::SHAR2 as u16 => Ok(Self::SHAR2),
            x if x == Self::SHAR3 as u16 => Ok(Self::SHAR3),
            x if x == Self::SHAR4 as u16 => Ok(Self::SHAR4),
            x if x == Self::SHAR5 as u16 => Ok(Self::SHAR5),
            x if x == Self::SIPR0 as u16 => Ok(Self::SIPR0),
            x if x == Self::SIPR1 as u16 => Ok(Self::SIPR1),
            x if x == Self::SIPR2 as u16 => Ok(Self::SIPR2),
            x if x == Self::SIPR3 as u16 => Ok(Self::SIPR3),
            x if x == Self::INTLEVEL0 as u16 => Ok(Self::INTLEVEL0),
            x if x == Self::INTLEVEL1 as u16 => Ok(Self::INTLEVEL1),
            x if x == Self::IR as u16 => Ok(Self::IR),
            x if x == Self::IMR as u16 => Ok(Self::IMR),
            x if x == Self::SIR as u16 => Ok(Self::SIR),
            x if x == Self::SIMR as u16 => Ok(Self::SIMR),
            x if x == Self::RTR0 as u16 => Ok(Self::RTR0),
            x if x == Self::RTR1 as u16 => Ok(Self::RTR1),
            x if x == Self::RCR as u16 => Ok(Self::RCR),
            x if x == Self::PTIMER as u16 => Ok(Self::PTIMER),
            x if x == Self::PMAGIC as u16 => Ok(Self::PMAGIC),
            x if x == Self::PHAR0 as u16 => Ok(Self::PHAR0),
            x if x == Self::PHAR1 as u16 => Ok(Self::PHAR1),
            x if x == Self::PHAR2 as u16 => Ok(Self::PHAR2),
            x if x == Self::PHAR3 as u16 => Ok(Self::PHAR3),
            x if x == Self::PHAR4 as u16 => Ok(Self::PHAR4),
            x if x == Self::PHAR5 as u16 => Ok(Self::PHAR5),
            x if x == Self::PSID0 as u16 => Ok(Self::PSID0),
            x if x == Self::PSID1 as u16 => Ok(Self::PSID1),
            x if x == Self::PMRU0 as u16 => Ok(Self::PMRU0),
            x if x == Self::PMRU1 as u16 => Ok(Self::PMRU1),
            x if x == Self::UIPR0 as u16 => Ok(Self::UIPR0),
            x if x == Self::UPORTR0 as u16 => Ok(Self::UPORTR0),
            x if x == Self::UPORTR1 as u16 => Ok(Self::UPORTR1),
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
        matches!(
            self,
            Reg::UIPR0
                | Reg::UIPR1
                | Reg::UIPR2
                | Reg::UIPR3
                | Reg::UPORTR0
                | Reg::UPORTR1
                | Reg::VERSIONR
        )
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
    /// Address of the SN_PORT register, index 0.
    PORT0 = 0x0004,
    /// Address of the SN_PORT register, index 1.
    PORT1 = 0x0005,
    /// Address of the SN_DHAR register, index 0.
    DHAR0 = 0x0006,
    /// Address of the SN_DHAR register, index 1.
    DHAR1 = 0x0007,
    /// Address of the SN_DHAR register, index 2.
    DHAR2 = 0x0008,
    /// Address of the SN_DHAR register, index 3.
    DHAR3 = 0x0009,
    /// Address of the SN_DHAR register, index 4.
    DHAR4 = 0x000A,
    /// Address of the SN_DHAR register, index 5.
    DHAR5 = 0x000B,
    /// Address of the SN_DIPR register, index 0.
    DIPR0 = 0x000C,
    /// Address of the SN_DIPR register, index 1.
    DIPR1 = 0x000D,
    /// Address of the SN_DIPR register, index 2.
    DIPR2 = 0x000E,
    /// Address of the SN_DIPR register, index 3.
    DIPR3 = 0x000F,
    /// Address of the SN_DPORT register, index 0.
    DPORT0 = 0x0010,
    /// Address of the SN_DPORT register, index 1.
    DPORT1 = 0x0011,
    /// Address of the SN_MSSR register, index 0.
    MSSR0 = 0x0012,
    /// Address of the SN_MSSR register, index 1.
    MSSR1 = 0x0013,
    /// Address of the SN_TOS register.
    TOS = 0x0015,
    /// Address of the SN_TTL register.
    TTL = 0x0016,
    /// Address of the SN_RXBUF_SIZE register.
    RXBUF_SIZE = 0x001E,
    /// Address of the SN_TXBUF_SIZE register.
    TXBUF_SIZE = 0x001F,
    /// Address of the SN_TX_FSR register, index 0.
    TX_FSR0 = 0x0020,
    /// Address of the SN_TX_FSR register, index 1.
    TX_FSR1 = 0x0021,
    /// Address of the SN_TX_RD register, index 0.
    TX_RD0 = 0x0022,
    /// Address of the SN_TX_RD register, index 1.
    TX_RD1 = 0x0023,
    /// Address of the SN_TX_WR register, index 0.
    TX_WR0 = 0x0024,
    /// Address of the SN_TX_WR register, index 1.
    TX_WR1 = 0x0025,
    /// Address of the SN_RX_RSR register, index 0.
    RX_RSR0 = 0x0026,
    /// Address of the SN_RX_RSR register, index 1.
    RX_RSR1 = 0x0027,
    /// Address of the SN_RX_RD register, index 0.
    RX_RD0 = 0x0028,
    /// Address of the SN_RX_RD register, index 1.
    RX_RD1 = 0x0029,
    /// Address of the SN_RX_WR register, index 0.
    RX_WR0 = 0x002A,
    /// Address of the SN_RX_WR register, index 1.
    RX_WR1 = 0x002B,
    /// Address of the SN_IMR register.
    IMR = 0x002C,
    /// Address of the SN_FRAG register, index 0.
    FRAG0 = 0x002D,
    /// Address of the SN_FRAG register, index 1.
    FRAG1 = 0x002E,
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
            x if x == Self::PORT0 as u16 => Ok(Self::PORT0),
            x if x == Self::PORT1 as u16 => Ok(Self::PORT1),
            x if x == Self::DHAR0 as u16 => Ok(Self::DHAR0),
            x if x == Self::DHAR1 as u16 => Ok(Self::DHAR1),
            x if x == Self::DHAR2 as u16 => Ok(Self::DHAR2),
            x if x == Self::DHAR3 as u16 => Ok(Self::DHAR3),
            x if x == Self::DHAR4 as u16 => Ok(Self::DHAR4),
            x if x == Self::DHAR5 as u16 => Ok(Self::DHAR5),
            x if x == Self::DIPR0 as u16 => Ok(Self::DIPR0),
            x if x == Self::DIPR1 as u16 => Ok(Self::DIPR1),
            x if x == Self::DIPR2 as u16 => Ok(Self::DIPR2),
            x if x == Self::DIPR3 as u16 => Ok(Self::DIPR3),
            x if x == Self::DPORT0 as u16 => Ok(Self::DPORT0),
            x if x == Self::DPORT1 as u16 => Ok(Self::DPORT1),
            x if x == Self::MSSR0 as u16 => Ok(Self::MSSR0),
            x if x == Self::MSSR1 as u16 => Ok(Self::MSSR1),
            x if x == Self::TOS as u16 => Ok(Self::TOS),
            x if x == Self::TTL as u16 => Ok(Self::TTL),
            x if x == Self::RXBUF_SIZE as u16 => Ok(Self::RXBUF_SIZE),
            x if x == Self::TXBUF_SIZE as u16 => Ok(Self::TXBUF_SIZE),
            x if x == Self::TX_FSR0 as u16 => Ok(Self::TX_FSR0),
            x if x == Self::TX_FSR1 as u16 => Ok(Self::TX_FSR1),
            x if x == Self::TX_RD0 as u16 => Ok(Self::TX_RD0),
            x if x == Self::TX_RD1 as u16 => Ok(Self::TX_RD1),
            x if x == Self::TX_WR0 as u16 => Ok(Self::TX_WR0),
            x if x == Self::TX_WR1 as u16 => Ok(Self::TX_WR1),
            x if x == Self::RX_RSR0 as u16 => Ok(Self::RX_RSR0),
            x if x == Self::RX_RSR1 as u16 => Ok(Self::RX_RSR1),
            x if x == Self::RX_RD0 as u16 => Ok(Self::RX_RD0),
            x if x == Self::RX_RD1 as u16 => Ok(Self::RX_RD1),
            x if x == Self::RX_WR0 as u16 => Ok(Self::RX_WR0),
            x if x == Self::RX_WR1 as u16 => Ok(Self::RX_WR1),
            x if x == Self::IMR as u16 => Ok(Self::IMR),
            x if x == Self::FRAG0 as u16 => Ok(Self::FRAG0),
            x if x == Self::FRAG1 as u16 => Ok(Self::FRAG1),
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
    /// assert_eq!(SnReg::PORT0.addr(), 0x0004);
    /// ```
    pub const fn addr(self) -> u16 {
        self as u16
    }

    /// Returns `true` if the register is read-only.
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_ll::SnReg;
    ///
    /// assert!(SnReg::SR.is_ro());
    /// assert!(!SnReg::MR.is_ro());
    /// ```
    pub const fn is_ro(self) -> bool {
        matches!(
            self,
            Self::SR
                | Self::TX_FSR0
                | Self::TX_FSR1
                | Self::TX_RD0
                | Self::TX_RD1
                | Self::RX_RSR0
                | Self::RX_RSR1
        )
    }
}
