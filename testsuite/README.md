# W5500 Testsuite
These are on-target tests for the W5500.

The purpose of these tests is to validate the `w5500-hl` crate with real hardware.

## Setup

Like all embedded development this takes quite a bit of effort to setup.

This uses some custom hardware, ([ambientsensor]), but this should be easy to adapt for another Cortex-M platform.

To make this work for another Cortex-M platform you should modify:

* `memory.x`
* `testsuite/Cargo.toml`
    * Replace the `stm32f0xx-hal` with the appropriate HAL for your device.
* `testsuite/tests/tests.rs`
    * The W5500 pin setup and mapping.
* `testsuite-assets/src/lib.rs`
    * Update addresses and ports to fit your network.

### Software Setup

I will assume you have rust (and your desired targets) installed.

* defmt-test (source: [app-template])
    * `cargo install flip-link`
    * `cargo install probe-run`

## How it works

There are two sides to this, the embedded W5500 and a peer for it to communicate with (your PC).

### W5500 Side

These tests run with [defmt-test], an embedded test framework.

The W5500 IP, Gateway, MAC, and subnet mask are all hard coded in the `testsuite-assets/src/lib.rs` file.

### Peer Side

To make the tests worthwhile the W5500 has to communicate with something.

The other side of this is another workspace member, `testsuite-peer`.

## Running

In one session start the W5500 side:
```bash
cargo test -p testsuite --target thumbv6m-none-eabi
```

Wait for the W5500 to start executing the first test, then in another session start the peer script:
```bash
cargo run -p testsuite-peer
```

[app-template]: https://github.com/knurling-rs/app-template
[defmt-test]: https://defmt.ferrous-systems.com/
[ambientsensor]: https://github.com/newam/ambientsensor
