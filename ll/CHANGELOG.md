# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.12.0] - 2023-10-02
### Changed
- Updated `embedded-hal` from `1.0.0-alpha.11` to `1.0.0-rc.1`.
- Updated `embedded-hal-async` from `0.2.0-alpha.2` to `1.0.0-rc.1`.

## [0.11.0] - 2023-08-06
### Added
- Added `#[must_use]` to functions in `w5500_ll::net`.
- Added support for `core::net` types with the `ip_in_core` feature.
- Added support for `embedded-hal` version `1.0.0-alpha.11` with the `eh1` feature.
- Added support for `embedded-hal-async` version `0.2.0-alpha.2` with the `eha0a` feature.
- Added an `aio` module with async traits.
- Added `sn_tx_ptrs` and `sn_rx_ptrs` to read socket buffer pointers in a single transaction.
- Added `From<Sn>` for all primitive signed and unsigned integers.
- Added `TryFrom<_> for Sn` for all primitive signed and unsigned integers.

### Changed
- Changed the `embedded-hal` version `0.2` feature name to `eh0`.
- Renamed the `blocking` module to `eh0`.
- Moved the `reset` function and `spi::MODE` constant into the `eh0` and `eh1` modules.
- Replaced public `octets` member on `net::Ipv4Addr` with an `octets()` method.

## [0.10.4] - 2022-07-14
### Added
- Added a new W5500 type for VDM transfers with an infallible GPIO and an infallible SPI bus.
- Added `defmt::Format` implementations for all register structs.
- Added the missing `opmd` field to the `Display` impl for `PhyCfg`.

## Fixed
- Fixed the `opmdc` field name in the `Display` impl for `PhyCfg`.  This field was incorrectly named `opmd`.

## [0.10.3] - 2022-04-27
### Added
- Added `sn_tx_buf` and `set_sn_rx_buf` methods to the `Registers` trait.

## [0.10.2] - 2022-04-10
### Added
- Added `Sn::iter()` which returns an iterator over all socket numbers.

## [0.10.1] - 2021-12-24
### Added
- Added a new W5500 type for VDM transfers with an infallible GPIO.
- Added `defmt::Format` for `Eui48Addr`.

## [0.10.0] - 2021-11-09
### Changed
- Changed the `socket` argument name to `sn` to match the type name.
- Updated `defmt` from `0.2` to `0.3`.

## [0.9.0] - 2021-11-07
### Added
- Added `defmt::Display` for all enumerations.
- Added `defmt::Display` for `fdm::W5500`, `vdm::W5500`, and `vdm::Error`.
- Added `Protocol::from_raw` to convert from a raw value in `const` contexts.
- Added `OperationMode::from_raw` to convert from a raw value in `const` contexts.
- Added `TryFrom<u8> for SocketCommand`.
- Added `SocketInterrupt.any_raised`.

### Changed
- Changed all field setters for registers to methods that return a modified structure.
  - Allows for functional programming styles.
  - Enables composing structures in `const` contexts.
- Changed edition from 2018 to 2021.
- Changed the return type of `PhyCfg::opmdc` from `Result<OperationMode, u8>` to `OperationMode`.
- Changed the register addresses in the `reg` module to `Reg` and `SnReg` enums.
- Changed the name of the `Socket` enum to `Sn`.

### Removed
- Removed `TryFrom<u8>` for `OperationMode`.

## [0.8.0] - 2021-05-08
### Added
- Added `Display` for `SocketInterruptMask`.
- Added `Display` for `SocketMode`.
- Added `Display` for `Mode`.

### Changed
- Changed the display formatting for `LinkStatus` and `SocketInterrupt`.

### Fixed
- Fixed `BufferSize::try_from` returning incorrect values for 4, 8, and 16 KB.

## [0.7.0] - 2021-02-23
### Changed
- Updated `defmt` dependency from 0.1.3 to 0.2.0.

## [0.6.2] - 2021-02-15
### Added
- Added a `bitmask` method to `Socket` for masking interrupts.
- Added `From<[u8; 4]> for Ipv4Addr`.
- Added `From<[u8; 6]> for Eui48Addr`.

## [0.6.1] - 2021-02-14
### Added
- Added a note about the assumed state of the chip select pin in the W5500 constructor.
- Added a `reset` function to perform a hardware reset of the W5500.

### Changed
- Changed the maintenance status from actively-developed to passively-maintained.

## [0.6.0] - 2021-02-08
### Added
- Implement `fmt::Display` for `SocketInterrupt`.
- Implement `defmt::Format` for `Ipv4Addr` and `SocketAddrV4`.
- Added a `free` method to the fixed data length implementation.

## Changed
- Changed `set_sn_ir` to accept any type that implements `Into<u8>`.
- Changed `embedded-hal` to a non-default feature.

## [0.5.1] - 2021-02-06
### Fixed
- Fixed documentation builds failing without the `embedded-hal` feature.

## [0.5.0] - 2021-02-06
### Added
- Added the `Default` trait for `DuplexStatus`, `LinkStatus`, and `SpeedStatus`.
- Added a `std` crate feature (disabled by default).
- Added the `From` trait to convert between `std::net` and `w5500_ll::net` types.

### Changed
- Changed fully qualified documentation links (e.g. `crate::`) to shorter links.
- Changed the `embedded-hal` dependency to an optional feature (enabled by default).
- Changed the maintenance status from experimental to actively-developed.

### Fixed
- Fixed incorrect documentation on `SocketMode::enable_mip6b` and `SocketMode::disable_mip6b`.

## [0.4.0] - 2021-01-30
### Added
- Added `impl From<Socket> for usize`.
- Added `sn_dest` and `set_sn_dest` methods.
- Added `net::SocketAddrV4`.

### Changed
- Changed `Registers::sn_sr` to return `Result<Result<SocketStatus, u8>, Self::Error>` instead of `Result<u8, Self::Error>` that would then need to be converted into a `SocketStatus`.
- Changed `Registers::sn_txbuf_size` and `Registers::sn_rxbuf_size` to return `Result<Result<BufferSize, u8>, Self::Error>` instead of `Result<u8, Self::Error>` that would then need to be converted into a `BufferSize`.
- Improved `set_sn_tx_buf` and `sn_rx_buf` examples.

## [0.3.0] - 2021-01-16
### Added
- Added additional methods to `w5500_ll::net::Ipv4Addr` from `std::net::Ipv4Addr`.
- Added `derive(hash)` to `Socket`.
- Added a public `spi` module with helpers and types for W5500 SPI transfers.
- Added a fixed data length implementation, `w5500_ll::blocking::fdm::W5500`.

### Changed
- Moved `w5500_ll::SPI_MODE` to `w5500_ll::spi::MODE`.
- `COMMON_BLOCK_OFFSET` is now public.
- Moved `w5500_ll::blocking::W5500` to `w5500_ll::blocking::vdm::W5500`

## [0.2.1] - 2021-01-03
### Fixed
- Fixed an overflow error that could occur in the `sn_rx_buf` and `set_sn_tx_buf` example code.

## [0.2.0] - 2021-01-03
### Added
- Added missing method to read from `sn_cr`.
- Added methods to `PhyCfg` to set `opmd`.

### Changed
- Improve `SocketStatus` documentation.
- Improve `SocketCommand` documentation.
- Removed the `ValueError` layer of abstraction. Functions that previously returned `Result<T, ValueError>` now return `Result<T, u8>`.

### Fixed
- Fixed `Registers::sn_dport` and `Registers::set_sn_dport` register address.
- Fixed `SocketInterruptMask::unmask_discon` and
  `SocketInterruptMask::mask_discon` operating on the incorrect field.
- Fixed logic inversion on `SocketInterruptMask::con_masked`, `SocketInterruptMask::discon_masked`, `SocketInterruptMask::recv_masked`, `SocketInterruptMask::timeout_masked`, and `SocketInterruptMask::sendok_masked`.
- Fixed `SocketInterrupt::clear_discon` operating on the incorrect field.
- Fixed `SocketMode::enable_mip6b` and `SocketMode::disable_mip6b` operating on the incorrect field.

## [0.1.0] - 2021-01-02
- Initial release

[Unreleased]: https://github.com/newAM/w5500-rs/compare/ll%2Fv0.12.0...HEAD
[0.12.0]: https://github.com/newAM/w5500-rs/compare/ll%2Fv0.11.0...ll%2Fv0.12.0
[0.11.0]: https://github.com/newAM/w5500-rs/compare/ll%2Fv0.10.4...ll%2Fv0.11.0
[0.10.4]: https://github.com/newAM/w5500-rs/compare/ll%2Fv0.10.3...ll%2Fv0.10.4
[0.10.3]: https://github.com/newAM/w5500-rs/compare/ll%2Fv0.10.2...ll%2Fv0.10.3
[0.10.2]: https://github.com/newAM/w5500-rs/compare/ll%2Fv0.10.1...ll%2Fv0.10.2
[0.10.1]: https://github.com/newAM/w5500-rs/compare/ll%2Fv0.10.0...ll%2Fv0.10.1
[0.10.0]: https://github.com/newAM/w5500-rs/compare/ll%2Fv0.9.0...ll%2Fv0.10.0
[0.9.0]: https://github.com/newAM/w5500-rs/compare/ll%2Fv0.8.0...ll%2Fv0.9.0
[0.8.0]: https://github.com/newAM/w5500-rs/compare/ll%2Fv0.7.0...ll%2Fv0.8.0
[0.7.0]: https://github.com/newAM/w5500-rs/compare/ll%2Fv0.6.2...ll%2Fv0.7.0
[0.6.2]: https://github.com/newAM/w5500-rs/compare/ll%2Fv0.6.1...ll%2Fv0.6.2
[0.6.1]: https://github.com/newAM/w5500-rs/compare/ll%2Fv0.6.0...ll%2Fv0.6.1
[0.6.0]: https://github.com/newAM/w5500-rs/compare/ll%2Fv0.5.1...ll%2Fv0.6.0
[0.5.1]: https://github.com/newAM/w5500-rs/compare/ll%2Fv0.5.0...ll%2Fv0.5.1
[0.5.0]: https://github.com/newAM/w5500-rs/compare/ll%2Fv0.4.0...ll%2Fv0.5.0
[0.4.0]: https://github.com/newAM/w5500-rs/compare/ll%2Fv0.3.0...ll%2Fv0.4.0
[0.3.0]: https://github.com/newAM/w5500-rs/compare/ll%2Fv0.2.1...ll%2Fv0.3.0
[0.2.1]: https://github.com/newAM/w5500-rs/compare/ll%2Fv0.2.0...ll%2Fv0.2.1
[0.2.0]: https://github.com/newAM/w5500-rs/compare/ll%2Fv0.1.0...ll%2Fv0.2.0
[0.1.0]: https://github.com/newAM/w5500-rs/releases/tag/ll%2Fv0.1.0
