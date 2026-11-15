use w5500_ll::Reg;

const REGS: [Reg; 48] = [
    Reg::MR,
    Reg::GAR0,
    Reg::GAR1,
    Reg::GAR2,
    Reg::GAR3,
    Reg::SUBR0,
    Reg::SUBR1,
    Reg::SUBR2,
    Reg::SUBR3,
    Reg::SHAR0,
    Reg::SHAR1,
    Reg::SHAR2,
    Reg::SHAR3,
    Reg::SHAR4,
    Reg::SHAR5,
    Reg::SIPR0,
    Reg::SIPR1,
    Reg::SIPR2,
    Reg::SIPR3,
    Reg::INTLEVEL0,
    Reg::INTLEVEL1,
    Reg::IR,
    Reg::IMR,
    Reg::SIR,
    Reg::SIMR,
    Reg::RTR0,
    Reg::RTR1,
    Reg::RCR,
    Reg::PTIMER,
    Reg::PMAGIC,
    Reg::PHAR0,
    Reg::PHAR1,
    Reg::PHAR2,
    Reg::PHAR3,
    Reg::PHAR4,
    Reg::PHAR5,
    Reg::PSID0,
    Reg::PSID1,
    Reg::PMRU0,
    Reg::PMRU1,
    Reg::UIPR0,
    Reg::UIPR1,
    Reg::UIPR2,
    Reg::UIPR3,
    Reg::UPORTR0,
    Reg::UPORTR1,
    Reg::PHYCFGR,
    Reg::VERSIONR,
];

#[test]
fn reg_conversion() {
    // every valid address must round-trip
    for reg in REGS {
        let addr: u16 = reg.addr();
        assert_eq!(
            Reg::try_from(addr),
            Ok(reg),
            "round-trip failed for addr {addr:#06X}"
        );
    }

    // unallocated addresses must be rejected
    for addr in 0x002F..=0x0038 {
        assert_eq!(Reg::try_from(addr), Err(addr));
    }
    assert_eq!(Reg::try_from(0x003A), Err(0x003A));
}
