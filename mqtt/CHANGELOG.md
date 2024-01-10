# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Removed
- Removed the `ip_in_core` and `std` features.  `Ipv4Addr` and `SocketAddrV4` are now provided by `core::net`.

## [0.3.0] - 2023-10-02
### Changed
- Updated `embedded-hal` from `1.0.0-alpha.11` to `1.0.0-rc.1`.
- Updated `embedded-hal-async` from `0.2.0-alpha.2` to `1.0.0-rc.1`.

## [0.2.0] - 2023-08-06
### Added
- Added support for `embedded-hal` version `1.0.0-alpha.11` with the `eh1` feature.

### Changed
- Updated `w5500-hl` dependency from `0.9.0` to `0.10.0`.
- Updated `w5500-tls` dependency from `0.1.2` to `0.2.0`.
- Changed the name of the `embedded-hal` feature to `eh0`.

## [0.1.2] - 2022-08-01
### Added
- Added a `p256-cm4` feature passthrough to `w5500-tls`.

### Changed
- Updated `w5500-tls` from `0.1.0` to `0.1.2`.

## [0.1.1] - 2022-05-14
### Changed
- Updated `w5500-tls` from `0.1.0-alpha.1` to `0.1.0`.

## [0.1.0] - 2022-05-04
- Initial release

[Unreleased]: https://github.com/newAM/w5500-rs/compare/mqtt%2Fv0.3.0...HEAD
[0.3.0]: https://github.com/newAM/w5500-rs/compare/mqtt%2Fv0.2.0...mqtt%2Fv0.3.0
[0.2.0]: https://github.com/newAM/w5500-rs/compare/mqtt%2Fv0.1.2...mqtt%2Fv0.2.0
[0.1.2]: https://github.com/newAM/w5500-rs/compare/mqtt%2Fv0.1.1...mqtt%2Fv0.1.2
[0.1.1]: https://github.com/newAM/w5500-rs/compare/mqtt%2Fv0.1.0...mqtt%2Fv0.1.1
[0.1.0]: https://github.com/newAM/w5500-rs/releases/tag/mqtt%2Fv0.1.0
