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

```console
$ cargo test -p testsuite --target thumbv6m-none-eabi
   Compiling w5500-hl v0.5.0 (/home/alex/git/w5500-hl-rs)
   Compiling testsuite-assets v0.1.0 (/home/alex/git/w5500-hl-rs/testsuite-assets)
   Compiling testsuite v0.1.0 (/home/alex/git/w5500-hl-rs/testsuite)
    Finished test [unoptimized + debuginfo] target(s) in 0.61s
     Running tests/test.rs (target/thumbv6m-none-eabi/debug/deps/test-9b01b52e675c07d7)
(HOST) INFO  flashing program (82.32 KiB)
(HOST) INFO  success!
────────────────────────────────────────────────────────────────────────────────
0 DEBUG Resetting the W5500
└─ test::tests::init @ tests/test.rs:137
1 DEBUG Polling for link up
└─ test::tests::init @ tests/test.rs:150
2 INFO  (1/4) running `would_block`...
└─ test::tests::__defmt_test_entry @ tests/test.rs:102
3 INFO  (2/4) running `tcp_server`...
└─ test::tests::__defmt_test_entry @ tests/test.rs:102
4 INFO  Polling for interrupt on Socket7
└─ test::poll_int @ tests/test.rs:80
5 INFO  Got interrupt on Socket7
└─ test::poll_int @ tests/test.rs:84
6 INFO  Polling for interrupt on Socket7
└─ test::poll_int @ tests/test.rs:80
7 INFO  Got interrupt on Socket7
└─ test::poll_int @ tests/test.rs:84
8 INFO  (3/4) running `tcp_client`...
└─ test::tests::__defmt_test_entry @ tests/test.rs:102
9 INFO  Polling for interrupt on Socket6
└─ test::poll_int @ tests/test.rs:80
10 INFO  Got interrupt on Socket6
└─ test::poll_int @ tests/test.rs:84
11 DEBUG Chunk 0

...

└─ test::tests::tcp_client @ tests/test.rs:247
102 INFO  Polling for interrupt on Socket6
└─ test::poll_int @ tests/test.rs:80
103 INFO  Got interrupt on Socket6
└─ test::poll_int @ tests/test.rs:84
104 DEBUG Chunk 31
└─ test::tests::tcp_client @ tests/test.rs:247
105 INFO  Polling for interrupt on Socket6
└─ test::poll_int @ tests/test.rs:80
106 INFO  Got interrupt on Socket6
└─ test::poll_int @ tests/test.rs:84
107 INFO  (4/4) running `udp`...
└─ test::tests::__defmt_test_entry @ tests/test.rs:102
108 INFO  Polling for interrupt on Socket5
└─ test::poll_int @ tests/test.rs:80
109 INFO  Got interrupt on Socket5
└─ test::poll_int @ tests/test.rs:84
110 INFO  Polling for interrupt on Socket5
└─ test::poll_int @ tests/test.rs:80
111 INFO  Got interrupt on Socket5
└─ test::poll_int @ tests/test.rs:84
112 INFO  Polling for interrupt on Socket5
└─ test::poll_int @ tests/test.rs:80
113 INFO  Got interrupt on Socket5
└─ test::poll_int @ tests/test.rs:84
114 INFO  all tests passed!
└─ test::tests::__defmt_test_entry @ tests/test.rs:102
────────────────────────────────────────────────────────────────────────────────
(HOST) INFO  device halted without error
```

Wait for the W5500 to start executing the first test, then in another session start the peer script:

```console
$ cargo run -p testsuite-peer
Sending HTTP GET request to: http://10.0.0.50
HTTP GET test PASSED
Listening on 10.0.0.3:8080
Chunk 0
Chunk 1
Chunk 2
Chunk 3
Chunk 4
Chunk 5
Chunk 6
Chunk 7
Chunk 8
Chunk 9
Chunk 10
Chunk 11
Chunk 12
Chunk 13
Chunk 14
Chunk 15
Chunk 16
Chunk 17
Chunk 18
Chunk 19
Chunk 20
Chunk 21
Chunk 22
Chunk 23
Chunk 24
Chunk 25
Chunk 26
Chunk 27
Chunk 28
Chunk 29
Chunk 30
Chunk 31
TCP client tests PASSED
Binding a UDP socket to 10.0.0.4:5657
Sending data to 10.0.0.50:5656
Done all tests
```

[app-template]: https://github.com/knurling-rs/app-template
[defmt-test]: https://defmt.ferrous-systems.com/
[ambientsensor]: https://github.com/newam/ambientsensor
