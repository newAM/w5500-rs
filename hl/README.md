![Maintenance](https://img.shields.io/badge/maintenance-experimental-blue.svg)
[![crates.io](https://img.shields.io/crates/v/w5500-hl.svg)](https://crates.io/crates/w5500-hl)
[![docs.rs](https://docs.rs/w5500-hl/badge.svg)](https://docs.rs/w5500-hl/)
[![CI](https://github.com/newAM/w5500-hl-rs/workflows/CI/badge.svg)](https://github.com/newAM/w5500-hl-rs/actions)

# w5500-hl

Platform agnostic rust driver for the [Wiznet W5500] internet offload chip.

This crate contains higher level socket operations, built ontop of my other
crate, [`w5500_ll`], which contains register accessors and networking data
types for the W5500.

## Warning

This crate is still in an early alpha state.
This has been published early to solicit feedback.

At the moment only UDP socket and TCP streams have been implemented.
TCP listeners have not yet been implemented.

[`w5500_ll`]: https://crates.io/crates/w5500-ll
[Wiznet W5500]: https://www.wiznet.io/product-item/w5500/
