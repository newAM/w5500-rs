# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

[Unreleased]: https://github.com/newAM/w5500-ll-rs/compare/v0.6.0...HEAD
[0.6.0]: https://github.com/newAM/w5500-ll-rs/compare/v0.5.1...v0.6.0
[0.5.1]: https://github.com/newAM/w5500-ll-rs/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/newAM/w5500-ll-rs/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/newAM/w5500-ll-rs/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/newAM/w5500-ll-rs/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/newAM/w5500-ll-rs/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/newAM/w5500-ll-rs/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/newAM/w5500-ll-rs/releases/tag/v0.1.0
