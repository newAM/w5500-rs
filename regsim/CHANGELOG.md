# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0-alpha.5] - 2021-11-07
### Changed
- Changed edition from 2018 to 2021.
- Reworked internal structure to remove limitations on calling `read`/`write` methods directly.
- Updated `w5500-ll` dependency from `0.7.0` to `0.8.0`.

### Removed
- Removed `W5500::new`, use `W5500::default` as a direct replacement.

## [0.1.0-alpha.4] - 2021-03-02
### Added
- Implemented `sir`.

### Changed
- Updated w5500-ll dependency to 0.7.0

## [0.1.0-alpha.3] - 2021-02-10
### Changed
- Updated w5500-ll dependency to 0.6.0

### Fixed
- Fixed logs displaying the incorrect block when accessing a socket RX buffer.

## [0.1.0-alpha.2] - 2021-02-06
### Added
- Added support for TCP listeners.

### Fixed
- Fixed writes to `sn_ir` such that they work as a write-1-clear.
- Fixed `sn_rx_wr`, `sn_rx_rd`, `sn_tx_rd`, and `sn_tx_wr` not initializing on the `Open` socket command.

## [0.1.0-alpha.1] - 2021-01-31
- Initial release

[Unreleased]: https://github.com/newAM/w5500-regsim-rs/compare/v0.1.0-alpha.5...HEAD
[0.1.0-alpha.5]: https://github.com/newAM/w5500-regsim-rs/compare/v0.1.0-alpha.4...v0.1.0-alpha.5
[0.1.0-alpha.4]: https://github.com/newAM/w5500-regsim-rs/compare/v0.1.0-alpha.3...v0.1.0-alpha.4
[0.1.0-alpha.3]: https://github.com/newAM/w5500-regsim-rs/compare/v0.1.0-alpha.2...v0.1.0-alpha.3
[0.1.0-alpha.2]: https://github.com/newAM/w5500-regsim-rs/compare/v0.1.0-alpha.1...v0.1.0-alpha.2
[0.1.0-alpha.1]: https://github.com/newAM/w5500-regsim-rs/releases/tag/v0.1.0-alpha.1
