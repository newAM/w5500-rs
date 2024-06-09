# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Changed
- Bumped the minimum `chrono` version from `0.4` to `0.4.32`.
- Updated `w5500-hl` dependency from `0.11.0` to `0.12.0`.

### Removed
- Removed the `ip_in_core` and `std` features.  `Ipv4Addr` and `SocketAddrV4` are now provided by `core::net`.

## [0.4.0] - 2023-10-02
### Changed
- Updated `embedded-hal` from `1.0.0-alpha.11` to `1.0.0-rc.1`.
- Updated `embedded-hal-async` from `0.2.0-alpha.2` to `1.0.0-rc.1`.
- Updated `w5500-hl` dependency from `0.10.0` to `0.11.0`.

## [0.3.0] - 2023-08-06
### Added
- Added support for `embedded-hal` version `1.0.0-alpha.11` with the `eh1` feature.

### Changed
- Changed the name of the `embedded-hal` feature to `eh0`.
- Removed used of functions deprecated in chrono version `0.4.23`.
- Updated `w5500-hl` dependency from `0.9.0` to `0.10.0`.

## [0.2.0] - 2022-05-21
### Added
- `impl TryFrom<chrono::naive::NaiveDateTime> for Timestamp`.
- Added exports for `chrono` and `time`.

### Changed
- Changed `Client.server` to public.
- Changed the server argument of `Client::new` from `Ipv4Addr` to `SocketAddrV4`.
- Changed the error type of `Timestamp` conversions from `()` to `TimestampError`.
- Moved socket setup from `Client.request` to `Client.setup_socket` to avoid repeat initialization.

### Removed
- Removed `Client::server` and `Client::set_server` methods.

## [0.1.0] - 2022-05-03
- Initial release

[Unreleased]: https://github.com/newAM/w5500-rs/compare/sntp%2Fv0.4.0...HEAD
[0.4.0]: https://github.com/newAM/w5500-rs/compare/sntp%2Fv0.3.0...sntp%2Fv0.4.0
[0.3.0]: https://github.com/newAM/w5500-rs/compare/sntp%2Fv0.2.0...sntp%2Fv0.3.0
[0.2.0]: https://github.com/newAM/w5500-rs/compare/sntp%2Fv0.1.0...sntp%2Fv0.2.0
[0.1.0]: https://github.com/newAM/w5500-rs/releases/tag/sntp%2Fv0.1.0
