# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased] - 2022-05-14
### Added
- Added `mdns::Client.ptr_question`.
- Added `Response.rr_count`.

### Changed
- Changed `Response.next_answer` to `Response.next_rr`.
- Changed `Answer` to `ResourceRecord`.
- Changed `Answer.rdata` type from `Option<Ipv4Addr>` to `RData`.

## [0.1.1] - 2022-05-09
### Fixed
- Fixed reading labels with more than one pointer.

## [0.1.0] - 2022-05-04
- Initial release

[Unreleased]: https://github.com/newAM/w5500-rs/compare/dns%2Fv0.1.1...HEAD
[0.1.1]: https://github.com/newAM/w5500-rs/compare/dns%2Fv0.1.0...dns%2Fv0.1.1
[0.1.0]: https://github.com/newAM/w5500-rs/releases/tag/dns%2Fv0.1.0
