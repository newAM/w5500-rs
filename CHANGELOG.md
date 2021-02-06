# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

[Unreleased]: https://github.com/newAM/w5500-hl-rs/compare/v0.1.0-alpha.2...HEAD
[0.1.0-alpha.2]: https://github.com/newAM/w5500-hl-rs/releases/tag/v0.1.0-alpha.2
[0.1.0-alpha.1]: https://github.com/newAM/w5500-hl-rs/releases/tag/v0.1.0-alpha.1
