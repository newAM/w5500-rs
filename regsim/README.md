# w5500-regsim

Register simulation for the [Wiznet W5500] internet offload chip.

This implements the [`w5500_ll::Registers`] trait using [`std::net`] sockets
to simulate the W5500 on your local PC.

This is a best-effort implementation to aid in development of application
code, not all features of the W5500 will be fully simulated.

## Feature Flags

All features are disabled by default.

* `async`: **Nightly only.** Implement asynchronous traits.

## Notes

This is in an early alpha state, there are many todos throughout the code.

### Not-implemented

* MR (Mode Register)
    * Wake on LAN
    * Ping block
    * PPPoE mode
    * Force ARP
* INTLEVEL (Interrupt Low Level Timer Register)
* IR (Interrupt Register)
* IMR (Interrupt Mask Register)
* GAR (Gateway IP Address Register)
* SUBR (Subnet Mask Register)
* SHAR (Source Hardware Address Register)
* SIPR (Source IP Address Register)
* INTLEVEL (Interrupt Low Level Timer Register)
* IR (Interrupt Register)
* IMR (Interrupt Mask Register)
* SIR (Socket Interrupt Register)
    * Partial; see SN_IR
* SIMR (Socket Interrupt Mask Register)
* RTR (Retry Time Register)
* RCR (Retry Count Register)
* PTIMER (PPP LCP Request Timer Register)
* PMAGIC (PPP LCP Magic Number Register)
* PHAR (PPP Destination MAC Address Register)
* PSID (PPP Session Identification Register)
* PMRU (PPP Maximum Segment Size Register)
* UIPR (Unreachable IP Address Register)
* UPORT (Unreachable Port Register)
* PHYCFGR (PHY Configuration Register)
* SN_MR (Socket n Mode Register)
* SN_IR (Socket n Interrupt Register)
    * DISCON
    * TIMEOUT
    * SENDOK
* SN_SR (Socket n Status Register)
    * SynSent
    * SynRecv
    * FinWait
    * Closing
    * TimeWait
    * CloseWait
    * LastAck
    * Macraw
* SN_MSSR (Socket n Maximum Segment Size Register)
* SN_TOS (Socket n IP TOS Register)
* SN_IMR (Socket n Interrupt Mask Register)
* SN_FRAG (Socket n Fragment Offset in IP Header Register)
* SN_KPALVTR (Socket n Keep Alive Timer Register)

Believe it or not that is not simply a list of all registers.

[Wiznet W5500]: https://www.wiznet.io/product-item/w5500/
[`std::net`]: https://doc.rust-lang.org/std/net/index.html
[`w5500-hl`]: https://crates.io/crates/w5500-hl
[`w5500_ll::Registers`]: https://docs.rs/w5500-ll/latest/w5500_ll/trait.Registers.html
