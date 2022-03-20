# w5500-dhcp

Simple DHCP client for the [Wiznet W5500] SPI internet offload chip.

## Warning

Please proceed with caution, and review the code before use in a production
environment.

This code was developed for one-off hobby projects.
It works for my network, but it likely has bugs.

### Limitations

* No support for rebinding
* No support for renewing

## Feature Flags

All features are disabled by default.

* `embedded-hal`: Passthrough to [`w5500-hl`].
* `std`: Passthrough to [`w5500-hl`].
* `defmt`: Enable logging with `defmt`.  Mutually exclusive with `log`.
  Also a passthrough to [`w5500-hl`].
* `log`: Enable logging with `log`.  Mutually exclusive with `defmt`.

[`std::net`]: https://doc.rust-lang.org/std/net/index.html
[`w5500-hl`]: https://github.com/newAM/w5500-hl-rs
[Wiznet W5500]: https://www.wiznet.io/product-item/w5500/