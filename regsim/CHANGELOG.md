# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- Added `#[derive(Debug, PartialEq)]` for `W5500`.
- Added `W5500.set_corpus_file` to record register reads to a file for fuzzing.

### Changed
- Change the `Register::Error` type from `std::io::Error` to `std::io::ErrorKind`.

### Fixed
- Fixed `Sn_RX_RSR` changing value between reading byte 0 and byte 1.
- Fixed a panic condition that occured when socket buffer pointers wrapped.

## [0.1.0-alpha.8] - 2022-04-10
### Added
- Added handling for `set_sn_rxbuf_size` and `set_sn_txbuf_size`.
- Added `W5500.socket_buffer_logging` to disable socket buffer logging at runtime.

### Changed
- Increased the verbosity of RX buffer overflow from `warn` to `error`.

## [0.1.0-alpha.7] - 2022-03-30
### Fixed
- Fixed UDP sockets binding to `dport` instead of `port`.
- Fixed UDP sockets binding to `0.0.0.0` instead of `sipr`.
- Fixed `SocketCommand::Send` not updating the `sn_tx_rd` pointer.
- Fixed `set_sn_ir` not clearing socket interrupts.
- Fixed overflow handling for socket RX buffers.

### Removed
- Removed an unused warning about the UDP socket buffer size.

## [0.1.0-alpha.6] - 2021-11-09
### Changed
- Updated `w5500-ll` dependency from `0.9.0` to `0.10.0`.

## [0.1.0-alpha.5] - 2021-11-07
### Changed
- Changed edition from 2018 to 2021.
- Reworked internal structure to remove limitations on calling `read`/`write` methods directly.
- Updated `w5500-ll` dependency from `0.7.0` to `0.9.0`.

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

[Unreleased]: https://github.com/newAM/w5500-rs/compare/regsim%2Fv0.1.0-alpha.8...HEAD
[0.1.0-alpha.8]: https://github.com/newAM/w5500-rs/compare/regsim%2Fv0.1.0-alpha.7...regsim%2Fv0.1.0-alpha.8
[0.1.0-alpha.7]: https://github.com/newAM/w5500-rs/compare/regsim%2Fv0.1.0-alpha.6...regsim%2Fv0.1.0-alpha.7
[0.1.0-alpha.6]: https://github.com/newAM/w5500-rs/compare/regsim%2Fv0.1.0-alpha.5...regsim%2Fv0.1.0-alpha.6
[0.1.0-alpha.5]: https://github.com/newAM/w5500-rs/compare/regsim%2Fv0.1.0-alpha.4...regsim%2Fv0.1.0-alpha.5
[0.1.0-alpha.4]: https://github.com/newAM/w5500-rs/compare/regsim%2Fv0.1.0-alpha.3...regsim%2Fv0.1.0-alpha.4
[0.1.0-alpha.3]: https://github.com/newAM/w5500-rs/compare/regsim%2Fv0.1.0-alpha.2...regsim%2Fv0.1.0-alpha.3
[0.1.0-alpha.2]: https://github.com/newAM/w5500-rs/compare/regsim%2Fv0.1.0-alpha.1...regsim%2Fv0.1.0-alpha.2
[0.1.0-alpha.1]: https://github.com/newAM/w5500-rs/releases/tag/regsim%2Fv0.1.0-alpha.1
