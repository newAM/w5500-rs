# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2021-02-27
## Added
- Added `is_state_tcp`, `is_state_udp`, and `is_state_closed` to the `Common` trait.

## Changed
- Updated w5500-ll dependency to 0.7.0

## [0.2.0] - 2021-02-10
### Added
- Added feature flags to pass through to w5500-ll.

### Changed
- Updated w5500-ll dependency to 0.6.0

## [0.1.0] - 2021-02-07
### Added
- Added a `tcp_disconnect` method.

### Changed
- Relocated example code using register simulation to the `w5500-regsim` repository.

### Fixed
- Fixed compilation for `no_std` targets when specifying the repository as a local path.

## [0.1.0-alpha.2] - 2021-02-06
### Added
- Added a `tcp_listen` method to the `Tcp` trait (formerly called `TcpListener`).
- Added an HTTP server example.
- Added examples to the README.

### Changed
- Changed the `TcpListener` trait name to `Tcp`.
- Changed the `CommonSocket` trait name to `Common`.

### Fixed
- Fixed `CommonSocket::local_addr` not returning the correct port.

## [0.1.0-alpha.1] - 2021-01-31
- Initial release

[Unreleased]: https://github.com/newAM/w5500-hl-rs/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/newAM/w5500-hl-rs/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/newAM/w5500-hl-rs/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/newAM/w5500-hl-rs/compare/v0.1.0-alpha.2...v0.1.0
[0.1.0-alpha.2]: https://github.com/newAM/w5500-hl-rs/compare/v0.1.0-alpha.1...v0.1.0-alpha.2
[0.1.0-alpha.1]: https://github.com/newAM/w5500-hl-rs/releases/tag/v0.1.0-alpha.1
