# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- Added support for `embedded-hal` version `1.0.0-alpha.9` with the `eh1` feature.

### Changed
- Changed the name of the `embedded-hal` feature to `eh0`.

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

[Unreleased]: https://github.com/newAM/w5500-rs/compare/sntp%2Fv0.2.0...HEAD
[0.2.0]: https://github.com/newAM/w5500-rs/compare/sntp%2Fv0.1.0...sntp%2Fv0.2.0
[0.1.0]: https://github.com/newAM/w5500-rs/releases/tag/sntp%2Fv0.1.0
