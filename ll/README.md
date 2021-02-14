![Maintenance](https://img.shields.io/badge/maintenance-passively--maintained-yellowgreen.svg)
[![crates.io](https://img.shields.io/crates/v/w5500-ll.svg)](https://crates.io/crates/w5500-ll)
[![docs.rs](https://docs.rs/w5500-ll/badge.svg)](https://docs.rs/w5500-ll/)
[![CI](https://github.com/newAM/w5500-ll-rs/workflows/CI/badge.svg)](https://github.com/newAM/w5500-ll-rs/actions)

# w5500-ll

Platform agnostic rust driver for the [Wiznet W5500] SPI internet offload
chip.

This is a low-level (ll) crate. The scope of this crate is:
1) Register accessors.
2) Networking data types.

Higher level functionality (such as socket operations) should be built
on-top of what is provided here.

## Example

Reading the VERSIONR register (a constant value).

```rust
use w5500_ll::{blocking::vdm::W5500, Registers};

let mut w5500 = W5500::new(spi, pin);
let version: u8 = w5500.version()?;
assert_eq!(version, 0x04);
```

## Feature Flags

All features are disabled by default.

* `defmt`: Enable formatting `Ipv4Addr` and `SocketAddrV4` with `defmt`.
* `embedded-hal`: Enables the [`blocking`] module which contains
  implementations of the [`Registers`] trait using the `embedded-hal` traits.
* `std`: Enables conversion between [`std::net`] and [`w5500_ll::net`] types.
  This is for testing purposes only, the `std` flag will not work on
  embedded systems because it uses the standard library.

## Related Crates

* [w5500-hl] - Higher level socket operations.
* [w5500-regsim] - Register simulation using [`std::net`].

[`std::net`]: https://doc.rust-lang.org/std/net/index.html
[w5500-hl]: https://github.com/newAM/w5500-hl-rs
[w5500-regsim]: https://github.com/newAM/w5500-regsim-rs
[Wiznet W5500]: https://www.wiznet.io/product-item/w5500/
[`blocking`]: https://docs.rs/w5500-ll/0.6.1/w5500_ll/blocking/index.html
[`Registers`]: https://docs.rs/w5500-ll/0.6.1/w5500_ll/trait.Registers.html
[`w5500_ll::net`]: https://docs.rs/w5500-ll/0.6.1/w5500_ll/net/index.html
