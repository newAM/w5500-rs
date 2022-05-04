# w5500-rs

[![CI](https://github.com/newAM/w5500-rs/workflows/CI/badge.svg)](https://github.com/newAM/w5500-rs/actions)

Embedded rust support for the [Wiznet W5500] SPI internet offload chip.

* [`w5500-ll`] contains low-level drivers, register setters & getters.
* [`w5500-hl`] contains higher-level drivers.
* [`w5500-regsim`] contains a simulation of the [`w5500-ll`] `Registers` trait.
* Other crates contain protocol implementations.

## Crates

| Name             | Crates.io                                                                                               | Documentation                                                                            |
|------------------|---------------------------------------------------------------------------------------------------------|------------------------------------------------------------------------------------------|
| [`w5500-dhcp`]   | [![crates.io](https://img.shields.io/crates/v/w5500-dhcp.svg)](https://crates.io/crates/w5500-dhcp)     | [![Documentation](https://docs.rs/w5500-dhcp/badge.svg)](https://docs.rs/w5500-dhcp)     |
| [`w5500-dns`]    | [![crates.io](https://img.shields.io/crates/v/w5500-dns.svg)](https://crates.io/crates/w5500-dns)       | [![Documentation](https://docs.rs/w5500-dns/badge.svg)](https://docs.rs/w5500-dns)       |
| [`w5500-hl`]     | [![crates.io](https://img.shields.io/crates/v/w5500-hl.svg)](https://crates.io/crates/w5500-hl)         | [![Documentation](https://docs.rs/w5500-hl/badge.svg)](https://docs.rs/w5500-hl)         |
| [`w5500-ll`]     | [![crates.io](https://img.shields.io/crates/v/w5500-ll.svg)](https://crates.io/crates/w5500-ll)         | [![Documentation](https://docs.rs/w5500-ll/badge.svg)](https://docs.rs/w5500-ll)         |
| [`w5500-mqtt`]   | [![crates.io](https://img.shields.io/crates/v/w5500-mqtt.svg)](https://crates.io/crates/w5500-mqtt)     | [![Documentation](https://docs.rs/w5500-mqtt/badge.svg)](https://docs.rs/w5500-mqtt)     |
| [`w5500-regsim`] | [![crates.io](https://img.shields.io/crates/v/w5500-regsim.svg)](https://crates.io/crates/w5500-regsim) | [![Documentation](https://docs.rs/w5500-regsim/badge.svg)](https://docs.rs/w5500-regsim) |
| [`w5500-sntp`]   | [![crates.io](https://img.shields.io/crates/v/w5500-sntp.svg)](https://crates.io/crates/w5500-sntp)     | [![Documentation](https://docs.rs/w5500-sntp/badge.svg)](https://docs.rs/w5500-sntp)     |
| [`w5500-tls`]    | [![crates.io](https://img.shields.io/crates/v/w5500-tls.svg)](https://crates.io/crates/w5500-tls)       | [![Documentation](https://docs.rs/w5500-tls/badge.svg)](https://docs.rs/w5500-tls)       |

[Wiznet W5500]: https://www.wiznet.io/product-item/w5500/
[`w5500-dhcp`]: https://github.com/newAM/w5500-rs/tree/main/dhcp
[`w5500-dns`]: https://github.com/newAM/w5500-rs/tree/main/dns
[`w5500-hl`]: https://github.com/newAM/w5500-rs/tree/main/hl
[`w5500-ll`]: https://github.com/newAM/w5500-rs/tree/main/ll
[`w5500-mqtt`]: https://github.com/newAM/w5500-rs/tree/main/mqtt
[`w5500-regsim`]: https://github.com/newAM/w5500-rs/tree/main/regsim
[`w5500-sntp`]: https://github.com/newAM/w5500-rs/tree/main/sntp
[`w5500-tls`]: https://github.com/newAM/w5500-rs/tree/main/tls
