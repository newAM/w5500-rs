use std::collections::HashMap;
use w5500_ll::reg;

lazy_static::lazy_static! {
    static ref COMMON_REGS_NAMES: HashMap<u16, &'static str> = {
        let mut m = HashMap::new();
        m.insert(reg::MR, "MR");
        m.insert(reg::GAR, "GAR[0]");
        m.insert(reg::GAR + 1, "GAR[1]");
        m.insert(reg::GAR + 2, "GAR[2]");
        m.insert(reg::GAR + 3, "GAR[3]");
        m.insert(reg::GAR + 4, "GAR[4]");
        m.insert(reg::GAR + 5, "GAR[5]");
        m.insert(reg::SUBR, "SUBR[0]");
        m.insert(reg::SUBR + 1, "SUBR[1]");
        m.insert(reg::SUBR + 2, "SUBR[2]");
        m.insert(reg::SUBR + 3, "SUBR[3]");
        m.insert(reg::SHAR, "SHAR[0]");
        m.insert(reg::SHAR + 1, "SHAR[1]");
        m.insert(reg::SHAR + 2, "SHAR[2]");
        m.insert(reg::SHAR + 3, "SHAR[3]");
        m.insert(reg::SIPR, "SIPR[0]");
        m.insert(reg::SIPR + 1, "SIPR[1]");
        m.insert(reg::SIPR + 2, "SIPR[2]");
        m.insert(reg::SIPR + 3, "SIPR[3]");
        m.insert(reg::INTLEVEL, "INTLEVEL[0]");
        m.insert(reg::INTLEVEL + 1, "INTLEVEL[1]");
        m.insert(reg::IR, "IR");
        m.insert(reg::IMR, "IMR");
        m.insert(reg::SIR, "SIR");
        m.insert(reg::SIMR, "SIMR");
        m.insert(reg::RTR, "RTR[0]");
        m.insert(reg::RTR + 1, "RTR[1]");
        m.insert(reg::RCR, "RCR");
        m.insert(reg::PTIMER, "PTIMER");
        m.insert(reg::PMAGIC, "PMAGIC");
        m.insert(reg::PHAR, "PHAR[0]");
        m.insert(reg::PHAR + 1, "PHAR[1]");
        m.insert(reg::PHAR + 2, "PHAR[2]");
        m.insert(reg::PHAR + 3, "PHAR[3]");
        m.insert(reg::PHAR + 4, "PHAR[4]");
        m.insert(reg::PHAR + 5, "PHAR[5]");
        m.insert(reg::PSID, "PSID[0]");
        m.insert(reg::PSID + 1, "PSID[1]");
        m.insert(reg::PMRU, "PMRU[0]");
        m.insert(reg::PMRU + 1, "PMRU[1]");
        m.insert(reg::UIPR, "UIPR[0]");
        m.insert(reg::UIPR + 1, "UIPR[1]");
        m.insert(reg::UIPR + 2, "UIPR[2]");
        m.insert(reg::UIPR + 3, "UIPR[3]");
        m.insert(reg::UPORTR, "UPORTR[0]");
        m.insert(reg::UPORTR + 1, "UPORTR[1]");
        m.insert(reg::PHYCFGR, "PHYCFGR");
        m.insert(reg::VERSIONR, "VERSIONR");
        m
    };
}

// Get the name of a common block register given the address.
pub fn common_reg_name(address: &u16) -> &'static str {
    COMMON_REGS_NAMES.get(address).unwrap_or(&"RESERVED")
}

lazy_static::lazy_static! {
    static ref SOCKET_REGS_NAME: HashMap<u16, &'static str> = {
        let mut m = HashMap::new();
        m.insert(reg::SN_MR, "SN_MR");
        m.insert(reg::SN_CR, "SN_CR");
        m.insert(reg::SN_IR, "SN_IR");
        m.insert(reg::SN_SR, "SN_SR");
        m.insert(reg::SN_PORT, "SN_PORT[0]");
        m.insert(reg::SN_PORT + 1, "SN_PORT[1]");
        m.insert(reg::SN_DHAR, "SN_DHAR[0]");
        m.insert(reg::SN_DHAR + 1, "SN_DHAR[1]");
        m.insert(reg::SN_DHAR + 2, "SN_DHAR[2]");
        m.insert(reg::SN_DHAR + 3, "SN_DHAR[3]");
        m.insert(reg::SN_DHAR + 4, "SN_DHAR[4]");
        m.insert(reg::SN_DHAR + 5, "SN_DHAR[5]");
        m.insert(reg::SN_DIPR, "SN_DIPR[0]");
        m.insert(reg::SN_DIPR + 1, "SN_DIPR[1]");
        m.insert(reg::SN_DIPR + 2, "SN_DIPR[2]");
        m.insert(reg::SN_DIPR + 3, "SN_DIPR[3]");
        m.insert(reg::SN_DPORT, "SN_DPORT[0]");
        m.insert(reg::SN_DPORT + 1, "SN_DPORT[1]");
        m.insert(reg::SN_MSSR, "SN_MSSR[0]");
        m.insert(reg::SN_MSSR + 1, "SN_MSSR[1]");
        m.insert(reg::SN_TOS, "SN_TOS");
        m.insert(reg::SN_TTL, "SN_TTL");
        m.insert(reg::SN_RXBUF_SIZE, "SN_RXBUF_SIZE");
        m.insert(reg::SN_TXBUF_SIZE, "SN_TXBUF_SIZE");
        m.insert(reg::SN_TX_FSR, "SN_TX_FSR[0]");
        m.insert(reg::SN_TX_FSR + 1, "SN_TX_FSR[1]");
        m.insert(reg::SN_TX_RD, "SN_TX_RD[0]");
        m.insert(reg::SN_TX_RD + 1, "SN_TX_RD[1]");
        m.insert(reg::SN_TX_WR, "SN_TX_WR[0]");
        m.insert(reg::SN_TX_WR + 1, "SN_TX_WR[1]");
        m.insert(reg::SN_RX_RSR, "SN_RX_RSR[0]");
        m.insert(reg::SN_RX_RSR + 1, "SN_RX_RSR[1]");
        m.insert(reg::SN_RX_RD, "SN_RX_RD[0]");
        m.insert(reg::SN_RX_RD + 1, "SN_RX_RD[1]");
        m.insert(reg::SN_RX_WR, "SN_RX_WR[0]");
        m.insert(reg::SN_RX_WR + 1, "SN_RX_WR[1]");
        m.insert(reg::SN_IMR, "SN_IMR");
        m.insert(reg::SN_FRAG, "SN_FRAG[0]");
        m.insert(reg::SN_FRAG + 1, "SN_FRAG[1]");
        m.insert(reg::SN_KPALVTR, "SN_KPALVTR");

        m
    };
}

// Get the name of a socket block register given the address.
pub fn socket_reg_name(address: &u16) -> &'static str {
    SOCKET_REGS_NAME.get(address).unwrap_or(&"RESERVED")
}
