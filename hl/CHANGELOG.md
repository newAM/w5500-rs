# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- Added support for `embedded-hal` version `1.0.0-alpha.10` with the `eh1` feature.

### Changed
- Changed the name of the `embedded-hal` feature to `eh0`.
- Changed `Seek<E>::seek` to `Seek::seek<E>`, moving the seek error generic from the `Seek` trait to the `seek` method.

## [0.9.0] - 2022-05-03
### Added
- Added `Hostname::new_unwrapped` to create a new hostname, panicking on invalid hostnames.
- Added a `Write` trait.

### Changed
- Split `Writer` into `TcpWriter` and `UdpWriter` that implement the `Write` trait.
- Split `Common::writer` into `Tcp::tcp_writer`, and `Udp::udp_writer`.
- Moved `SeekFrom`, `Seek`, `Read`, and `Write` into an `io` module.
- Simplified the generics of the `Read` trait from `<'a, W5500: Registers>` to `<E>`.

### Removed
- Removed `non_exhaustive` from `Error`.

## [0.8.0] - 2022-04-10
### Added
- Added APIs for streaming IO to the socket buffers.
- Added an `Error` type to support `read_exact` and `write_all` methods.
- Added a `UdpHeader` type.
- Added a `Hostname` type for use with DNS and DHCP.

### Changed
- Changed all methods that return a `usize` of the bytes read / written to return a `u16` instead.  This helps avoid lossy conversions from `usize` to `u16` in some applications.
- Changed `udp_peek_from` to return `(u16, UdpHeader)` instead of `(u16, SocketAddrV4)`.
- Changed `udp_peek_from_header` to return `UdpHeader` instead of `(u16, SocketAddrV4)`.

### Removed
- Removed the `nb` dependency to consolidate error types.
  - To update replace `nb::block!` with `w5500_hl::block!` for these methods:
    - `udp_peek_from_header`
    - `udp_peek_from`
    - `udp_recv_from`

## [0.7.1] - 2021-12-24
### Added
- Added `send_if_free` and `send_to_if_free` to the `Udp` trait.
- Added a re-export for `nb`.

## [0.7.0] - 2021-11-09
### Changed
- Updated the `w5500-ll` dependency from `0.9.0` to `0.10.0`.
  - This updates `defmt` from `0.2` to `0.3`

## [0.6.0] - 2021-11-07
### Changed
- Changed edition from 2018 to 2021.
- Updated the `w5500-ll` dependency from `0.8.0` to `0.9.0`.
- Changed RSR math to reduce the size of debug builds.

## [0.5.0] - 2021-10-18
### Changed
- Removed the generic parameter from the `Udp`, `Tcp`, and `Common` traits.

## [0.4.0] - 2021-05-08
### Changed
- Updated w5500-ll dependency from 0.7.0 to 0.8.0.

## [0.3.0] - 2021-02-27
## Added
- Added `is_state_tcp`, `is_state_udp`, and `is_state_closed` to the `Common` trait.

## Changed
- Updated w5500-ll dependency from 0.6.0 to 0.7.0.

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

[Unreleased]: https://github.com/newAM/w5500-rs/compare/hl%2Fv0.9.0...HEAD
[0.9.0]: https://github.com/newAM/w5500-rs/compare/hl%2Fv0.8.0...hl%2Fv0.9.0
[0.8.0]: https://github.com/newAM/w5500-rs/compare/hl%2Fv0.7.1...hl%2Fv0.8.0
[0.7.1]: https://github.com/newAM/w5500-rs/compare/hl%2Fv0.7.0...hl%2Fv0.7.1
[0.7.0]: https://github.com/newAM/w5500-rs/compare/hl%2Fv0.6.0...hl%2Fv0.7.0
[0.6.0]: https://github.com/newAM/w5500-rs/compare/hl%2Fv0.5.0...hl%2Fv0.6.0
[0.5.0]: https://github.com/newAM/w5500-rs/compare/hl%2Fv0.4.0...hl%2Fv0.5.0
[0.4.0]: https://github.com/newAM/w5500-rs/compare/hl%2Fv0.3.0...hl%2Fv0.4.0
[0.3.0]: https://github.com/newAM/w5500-rs/compare/hl%2Fv0.2.0...hl%2Fv0.3.0
[0.2.0]: https://github.com/newAM/w5500-rs/compare/hl%2Fv0.1.0...hl%2Fv0.2.0
[0.1.0]: https://github.com/newAM/w5500-rs/compare/hl%2Fv0.1.0-alpha.2...hl%2Fv0.1.0
[0.1.0-alpha.2]: https://github.com/newAM/w5500-rs/compare/hl%2Fv0.1.0-alpha.1...hl%2Fv0.1.0-alpha.2
[0.1.0-alpha.1]: https://github.com/newAM/w5500-rs/releases/tag/hl%2Fv0.1.0-alpha.1
