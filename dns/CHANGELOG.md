# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.0] - 2024-06-09
### Changed
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
- Updated `w5500-hl` dependency from `0.9.0` to `0.10.0`.

## [0.2.0] - 2022-05-14
### Added
- Added `mdns::Client.ptr_question`.
- Added `Response.rr_count`.

### Changed
- Changed `Answer.rdata` type from `Option<Ipv4Addr>` to `RData`.
- Renamed `Answer` to `ResourceRecord`.
- Renamed `Response.next_answer` to `Response.next_rr`.

## [0.1.1] - 2022-05-09
### Fixed
- Fixed reading labels with more than one pointer.

## [0.1.0] - 2022-05-04
- Initial release

[Unreleased]: https://github.com/newAM/w5500-rs/compare/dns%2Fv0.5.0...HEAD
[0.5.0]: https://github.com/newAM/w5500-rs/compare/dns%2Fv0.4.0...dns%2Fv0.5.0
[0.4.0]: https://github.com/newAM/w5500-rs/compare/dns%2Fv0.3.0...dns%2Fv0.4.0
[0.3.0]: https://github.com/newAM/w5500-rs/compare/dns%2Fv0.2.0...dns%2Fv0.3.0
[0.2.0]: https://github.com/newAM/w5500-rs/compare/dns%2Fv0.1.1...dns%2Fv0.2.0
[0.1.1]: https://github.com/newAM/w5500-rs/compare/dns%2Fv0.1.0...dns%2Fv0.1.1
[0.1.0]: https://github.com/newAM/w5500-rs/releases/tag/dns%2Fv0.1.0
