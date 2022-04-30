# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- Added a small cache for DHCP options to reduce redundant socket IO.
- Added support for renewing and rebinding an existing lease.
- Added `Client.set_timeout` to set the timeout duration.

### Changed
- Decreased link-up polling interval from 2 seconds to 1 second.
- Changed the default timeout from 11 seconds to 5 seconds.

### Removed
- Removed `Client.is_bound`, use `Client.has_lease` instead.

## [0.3.0] - 2022-04-10
### Added
- Added `Client.setup_socket` to setup socket the socket.
- Added `Client.dns` and `Client.ntp` to get the DNS and NTP server IPs provided by the DHCP server.

### Changed
- Changed `w5500_dhcp::Error` to `w5500_hl::Error`.
- Changed DHCP packet processing from a local buffer to streaming from the W5500 socket buffer.
  - `Client::new` no longer accepts a `buf` argument.
- Changed the `Dhcp` structure name to `Client`.
  - The structure previously named `Client` has been removed.
- Changed the type of the `hostname` argument in `Client::new` from an unvalidated `&str` to a validated `Hostname`.
- Changed hostnames in `Client` from `'static` to a generic lifetime.
- Changed `State` from public to private.
- Improved handling for a physically broken link.
- Renamed `DHCP_DESTINATION_PORT` to `DST_PORT`.
- Renamed `DHCP_SOURCE_PORT` to `SRC_PORT`.

### Removed
- Removed `on_recv_interrupt`, use `process` instead.
- Removed `poll`, use `process` instead.

### Fixed
- Fixed handling for receiving multiple consecutive packets.

## [0.2.0] - 2022-01-02
### Changed
- DHCP lease time is de-rated by 12%.

## [0.1.0] - 2021-12-24
- Initial release

[Unreleased]: https://github.com/newAM/w5500-rs/compare/dhcp%2Fv0.3.0...HEAD
[0.3.0]: https://github.com/newAM/w5500-rs/compare/dhcp%2Fv0.2.0...dhcp%2Fv0.3.0
[0.2.0]: https://github.com/newAM/w5500-rs/compare/dhcp%2Fv0.1.0...dhcp%2Fv0.2.0
[0.1.0]: https://github.com/newAM/w5500-rs/releases/tag/dhcp%2Fv0.1.0
