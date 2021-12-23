![Maintenance](https://img.shields.io/badge/maintenance-experimental-blue.svg)
[![crates.io](https://img.shields.io/crates/v/w5500-hl.svg)](https://crates.io/crates/w5500-hl)
[![docs.rs](https://docs.rs/w5500-hl/badge.svg)](https://docs.rs/w5500-hl/)
[![CI](https://github.com/newAM/w5500-hl-rs/workflows/CI/badge.svg)](https://github.com/newAM/w5500-hl-rs/actions)

# w5500-hl

Platform agnostic rust driver for the [Wiznet W5500] internet offload chip.

This crate contains higher level (hl) socket operations, built on-top of my
other crate, [w5500-ll], which contains register accessors, and networking
data types for the W5500.

## Design

There are no separate socket structures.
The [`Tcp`] and [`Udp`] traits provided in this crate simply extend the
[`Registers`] trait provided in [w5500-ll].
This makes for a less ergonomic API, but a much more portable API because
there are no mutexes or runtime checks to enable socket structures to share
ownership of the underlying W5500 device.

You will likely want to wrap up the underlying structure that implements
the [`Registers`], [`Tcp`], and [`Udp`] traits to provide separate socket
structures utilizing whatever Mutex is available for your platform / RTOS.

## Feature Flags

All features are disabled by default.

* `defmt`: Passthrough to [w5500-ll].
* `embedded-hal`: Passthrough to [w5500-ll].
* `std`: Passthrough to [w5500-ll].

## Examples

UDP sockets

```rust
use w5500_hl::ll::{
    net::{Ipv4Addr, SocketAddrV4},
    Registers,
    Sn::Sn0,
};
use w5500_hl::Udp;

// open Sn0 as a UDP socket on port 1234
w5500.udp_bind(Sn0, 1234)?;

// send 4 bytes to 192.168.2.4:8080, and get the number of bytes transmitted
let data: [u8; 4] = [0, 1, 2, 3];
let destination = SocketAddrV4::new(Ipv4Addr::new(192, 168, 2, 4), 8080);
let tx_bytes = w5500.udp_send_to(Sn0, &data, &destination)?;
```

TCP streams (client)

```rust
use w5500_hl::ll::{
    net::{Ipv4Addr, SocketAddrV4},
    Registers, Sn,
};
use w5500_hl::Tcp;

const MQTT_SOCKET: Sn = Sn::Sn0;
const MQTT_SOURCE_PORT: u16 = 33650;
const MQTT_SERVER: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(192, 168, 2, 10), 1883);

// initiate a TCP connection to a MQTT server
w5500.tcp_connect(MQTT_SOCKET, MQTT_SOURCE_PORT, &MQTT_SERVER)?;
```

TCP listeners (server)

```rust
use w5500_hl::ll::{
    net::{Ipv4Addr, SocketAddrV4},
    Registers, Sn,
};
use w5500_hl::Tcp;

const HTTP_SOCKET: Sn = Sn::Sn1;
const HTTP_PORT: u16 = 80;

// serve HTTP
w5500.tcp_listen(HTTP_SOCKET, HTTP_PORT)?;
```

## Related Crates

* [w5500-ll] - Low level W5500 register accessors.
* [w5500-regsim] - Register simulation using [`std::net`].

[`Registers`]: https://docs.rs/w5500-ll/latest/w5500_ll/trait.Registers.html
[`std::net`]: https://doc.rust-lang.org/std/net/index.html
[w5500-ll]: https://github.com/newAM/w5500-ll-rs
[w5500-regsim]: https://github.com/newAM/w5500-regsim-rs
[Wiznet W5500]: https://www.wiznet.io/product-item/w5500/
[`Tcp`]: https://docs.rs/w5500-hl/0.7.0/w5500_hl/trait.Tcp.html
[`Udp`]: https://docs.rs/w5500-hl/0.7.0/w5500_hl/trait.Udp.html
