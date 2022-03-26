# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Changed
- Changed `w5500_dhcp::Error` for `w5500_hl::Error`.
- Changed DHCP packet processing from a local buffer to streaming from the W5500 socket buffer.
  - `Client::new` no longer accepts a `buf` argument.

## [0.2.0] - 2022-01-02
### Changed
- DHCP lease time is de-rated by 12%.

## [0.1.0] - 2021-12-24
- Initial release

[Unreleased]: https://github.com/newAM/w5500-rs/compare/dhcp%2Fv0.2.0...HEAD
[0.2.0]: https://github.com/newAM/w5500-rs/compare/dhcp%2Fv0.1.0...dhcp%2Fv0.2.0
[0.1.0]: https://github.com/newAM/w5500-rs/releases/tag/dhcp%2Fv0.1.0
