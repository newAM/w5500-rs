# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.2] - 2022-08-01
### Added
- Added a p256 implementation optimized for the Cortex-M4 CPU with the `p256-cm4` feature.

## [0.1.1] - 2022-07-26
### Fixed
- Fixed a race condition where the client would fail to send finished before the server connection timeout expired.

## [0.1.0] - 2022-05-14
### Changed
- Minor tweaks to improve performance and code size.
- Updated `p256` from `0.10.1` to `0.11.0`.

## [0.1.0-alpha.1] - 2022-05-04
- Initial release

[Unreleased]: https://github.com/newAM/w5500-rs/compare/tls%2Fv0.1.2...HEAD
[0.1.2]: https://github.com/newAM/w5500-rs/compare/tls%2Fv0.1.1...tls%2Fv0.1.2
[0.1.1]: https://github.com/newAM/w5500-rs/compare/tls%2Fv0.1.0...tls%2Fv0.1.1
[0.1.0]: https://github.com/newAM/w5500-rs/releases/tag/tls%2Fv0.1.0-alpha.1...tls%2Fv0.1.0
[0.1.0-alpha.1]: https://github.com/newAM/w5500-rs/releases/tag/tls%2Fv0.1.0-alpha.1
