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

[Wiznet W5500]: https://docs.wiznet.io/Product/iEthernet/W5500/overview
[`w5500-dhcp`]: https://github.com/newAM/w5500-rs/tree/main/dhcp
[`w5500-dns`]: https://github.com/newAM/w5500-rs/tree/main/dns
[`w5500-hl`]: https://github.com/newAM/w5500-rs/tree/main/hl
[`w5500-ll`]: https://github.com/newAM/w5500-rs/tree/main/ll
[`w5500-mqtt`]: https://github.com/newAM/w5500-rs/tree/main/mqtt
[`w5500-regsim`]: https://github.com/newAM/w5500-rs/tree/main/regsim
[`w5500-sntp`]: https://github.com/newAM/w5500-rs/tree/main/sntp
[`w5500-tls`]: https://github.com/newAM/w5500-rs/tree/main/tls

## Fork delta

This fork adds an async, fixed-size UDP fast path for hard-real-time use. It is
kept deliberately small so it can be re-synced with upstream.

- `w5500-hl` gains `fast_udp`: a `FastUdp` trait, plus `FastUdpAsync` behind the
  new `eha1` feature, generated from the same source by `maybe-async-cfg`.
- Receive costs **four** SPI transactions instead of six: `sn_rx_ptrs` reads
  `Sn_RX_RSR` and `Sn_RX_RD` together in one 4-byte read (they are adjacent
  registers), and the 8-byte header and the payload are read in a single
  transaction. Send costs **four** instead of five.
- `udp_send_to` costs one extra transaction over `udp_send`: `set_sn_dest`
  issues a single combined 6-byte write to `Sn_DIPR`, since the IP and port
  registers are contiguous.
- `udp_recv_exact` and `udp_send_exact` return `Error::UnexpectedLength` or
  `Error::OutOfMemory` rather than silently truncating a fixed-size datagram.
- Changes to existing upstream files: `hl/src/lib.rs` (module declaration,
  feature doc, one `Error` variant, and `#![allow(async_fn_in_trait)]`
  mirroring `ll/src/lib.rs`), `hl/src/udp.rs` (three items widened to
  `pub(crate)`), `hl/Cargo.toml` (the new `eha1` feature and its
  dependencies), `tls/src/alert.rs` (one match arm, required because `Error`
  is not `#[non_exhaustive]`), `ll/src/eh1/vdm.rs` (the `{new, free}` impl
  block un-bounded so an async-only SPI device can construct a `W5500` at
  all — without this the async layer is unusable), and `.gitignore`.
- Upstream's `Udp` trait, TCP, DHCP, DNS, MQTT and SNTP are untouched.
- `w5500-regsim` already implemented `w5500_ll::aio::Registers`, so no shim
  was needed: the async path is validated on the host against real UDP
  sockets.
