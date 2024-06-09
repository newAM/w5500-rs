# w5500-dhcp

DHCP client for the [Wiznet W5500] SPI internet offload chip.

## Warning

Please review the code before use in a production environment.
This code has been tested, but only with a single DHCP server.

## Feature Flags

All features are disabled by default.

* `eh0`: Passthrough to [`w5500-hl`].
* `eh1`: Passthrough to [`w5500-hl`].
* `defmt`: Enable logging with `defmt`. Also a passthrough to [`w5500-hl`].
* `log`: Enable logging with `log`.

[`w5500-hl`]: https://crates.io/crates/w5500-hl
[Wiznet W5500]: https://docs.wiznet.io/Product/iEthernet/W5500/overview
