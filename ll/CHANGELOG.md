# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Changed
- Improve `SocketStatus` documentation.
- Improve `SocketCommand` documentation.
- Removed the `ValueError` layer of abstraction.
  Functions that previously returned `Result<T, ValueError>` now return
  `Result<T, u8>`.

### Fixed
- Fixed `Registers::sn_dport` and `Registers::set_sn_dport` register address.
- Fixed `SocketInterruptMask::unmask_discon` and
  `SocketInterruptMask::mask_discon` operating on the incorrect field.
- Fixed logic inversion on `SocketInterruptMask::con_masked`,
  `SocketInterruptMask::discon_masked`, `SocketInterruptMask::recv_masked`,
  `SocketInterruptMask::timeout_masked`, and `SocketInterruptMask::sendok_masked`.
- Fixed `SocketInterrupt::clear_discon` operating on the incorrect field.
- Fixed `SocketMode::enable_mip6b` and `SocketMode::disable_mip6b` operating on
  the incorrect field.

## [0.1.0] - 2021-01-02
- Initial release

[Unreleased]: https://github.com/newAM/w5500-ll-rs/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/newAM/w5500-ll-rs/releases/tag/v0.1.0
