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
use w5500_ll::{eh1::vdm::W5500, Registers};

let mut w5500 = W5500::new(spi);
let version: u8 = w5500.version()?;
assert_eq!(version, 0x04);
```

## Feature Flags

All features are disabled by default.

* `async`: **Nightly only.** Enables the asynchronous [`aio::Registers`]
  trait.
* `defmt`: Enable formatting most types with `defmt`.
* `eh0`: Enables the [`eh0`] module which contains
  implementations of the [`Registers`] trait
  using the `embedded-hal` version 0.2 traits.
* `eh1`: Enables the [`eh1`] module which contains
  implementations of the [`Registers`] trait
  using the `embedded-hal` version 1 traits.
* `eha0a`: **Nightly only.**
  Implements the [`aio::Registers`] trait for types in the [`eh1`] module
  using the `embedded-hal-async` alpha traits.
* `std`: Enables conversion between [`std::net`] and [`w5500_ll::net`] types.
  This is for testing purposes only, the `std` flag will not work on
  embedded systems because it uses the standard library.

[`std::net`]: https://doc.rust-lang.org/std/net/index.html
[Wiznet W5500]: https://www.wiznet.io/product-item/w5500/
[`aio::Registers`]: https://docs.rs/w5500-ll/latest/w5500_ll/aio/trait.Registers.html
[`aio`]: https://docs.rs/w5500-ll/latest/w5500_ll/aio/index.html
[`eh0`]: https://docs.rs/w5500-ll/latest/w5500_ll/eh0/index.html
[`eh1`]: https://docs.rs/w5500-ll/latest/w5500_ll/eh1/index.html
[`Registers`]: https://docs.rs/w5500-ll/latest/w5500_ll/trait.Registers.html
[`w5500_ll::net`]: https://docs.rs/w5500-ll/latest/w5500_ll/net/index.html
