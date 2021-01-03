# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Changed
- Improve `SocketStatus` documentation.
- Improve `SocketCommand` documentation.

### Fixed
- Fixed `sn_dport` and `set_sn_dport` register address.
- Fixed `unmask_discon` and `mask_discon` operating on the incorrect field.
- Fixed logic inversion on `con_masked`, `discon_masked`, `recv_masked`,
  `timeout_masked`, and `sendok_masked`.

## [0.1.0] - 2021-01-02
- Initial release

[Unreleased]: https://github.com/newAM/w5500-ll-rs/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/newAM/w5500-ll-rs/releases/tag/v0.1.0
