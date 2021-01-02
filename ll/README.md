![Maintenance](https://img.shields.io/badge/maintenance-experimental-blue.svg)
[![crates.io](https://img.shields.io/crates/v/w5500-ll.svg)](https://crates.io/crates/w5500-ll)
[![docs.rs](https://docs.rs/w5500-ll/badge.svg)](https://docs.rs/w5500-ll/)
[![CI](https://github.com/newAM/w5500-ll-rs/workflows/CI/badge.svg)](https://github.com/newAM/w5500-ll-rs/actions)

# w5500-ll

Platform agnostic rust driver for the [Wiznet W5500] SPI internet offload
chip.

This is a low-level (ll) crate, specifically limited in scope to register
accessors only.
Higher level funcitonality (such as socket operations) should be built
ontop of what is provided here.

[Wiznet W5500]: https://www.wiznet.io/product-item/w5500/
