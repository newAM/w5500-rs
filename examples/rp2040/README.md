# RP2040 hardware diagnostics

Hardware acceptance tests for the `w5500-hl` fast UDP path, on an RP2040 with a
W5500 on SPI0 and **no debug probe**. This crate is a standalone workspace — it
is not built by the library's `cargo test`/`cargo build`, and it keeps the
embassy dependency tree out of the library.

If you are holding the board and wondering what to run first: `identify`, then
read [Running](#running).

The MCP251xFD controllers on this board are on SPI1; nothing here touches them.

## Board wiring

| Signal | RP2040 pin |
|---|---|
| SPI0 SCK | GPIO 18 |
| SPI0 MOSI | GPIO 19 |
| SPI0 MISO | GPIO 20 |
| W5500 CS | GPIO 17 |

**MOSI on GP19, MISO on GP20 — this is the only assignment SPI0 can drive.**
The RP2040 pin mux fixes SPI0 TX to GP19/GP23 and SPI0 RX to GP16/GP20; the
inverse assignment is not a wiring mistake you can make and still have the
peripheral work at all. `identify` is what settles this in practice: `VERSIONR`
is a constant `0x04`, so if the data lines are swapped (or one of them is
floating) you read back `0x00` or `0xFF`, never `0x04`.

## SPI clock

`common::SPI_FREQUENCY_HZ` is **31.25 MHz**, not 50 MHz. The RP2040 derives SPI
frequency as `clk_peri / (CPSDVSR * (1 + SCR))` with `CPSDVSR` even, so from a
125 MHz `clk_peri` the reachable rates near the top are 62.5 MHz and 31.25 MHz
with **nothing between**. `embassy-rp` rounds a requested frequency **down** to
whatever the divider can produce, silently — asking for 50 MHz gives 31.25 MHz
with no error and no log line.

`spiclock` exists because of this: it reports the rate actually measured on the
wire, not the rate requested, and sweeps a range of candidate rates to find the
highest one that still round-trips 188-byte frames cleanly. Do not assume the
requested value in the source is what the peripheral produced — measure it.

## Network

Device `192.168.0.10/24`, gateway `192.168.0.1`, socket 6, port 49200. Peer
(the simulator, or `host/sim.py`) at `192.168.0.1:49200`. Inbound datagrams are
exactly 180 bytes; outbound are exactly 32 bytes. `echo`, `endian` and
`latency` fill those 32 bytes with the production 8xf32 motor-command format.
**`soak` does not** — see [`soak`'s diagnostic reply](#soaks-diagnostic-reply)
below.

Only `soak` and `bufsize` size the socket buffers (socket 6 gets 4 KiB RX / 2
KiB TX, every other socket zeroed — see [Design notes](#design-notes)).
`echo`, `endian` and `latency` never touch `Sn_RXBUF_SIZE`/`Sn_TXBUF_SIZE`, so
they run on the chip's 2 KiB-per-socket default.

## Binaries

Run them in this order — each rung's verdict is only meaningful once the ones
above it pass, and each needs progressively more of the rig connected.

| Binary | Needs | What it proves |
|---|---|---|
| `identify` | SPI wiring only | `VERSIONR == 0x04` and a socket register round trip — the data lines carry arbitrary bytes, not a stuck level. **Run first on new hardware.** |
| `spiclock` | SPI wiring only | The rate actually achieved vs. requested, and the highest rate that stays clean over 64 bulk round trips. |
| `link` | Ethernet cable | PHY link/speed/duplex, and the network registers (`SHAR`/`SIPR`/`SUBR`/`GAR`) read back from the chip rather than trusted from the write. |
| `bufsize` | SPI wiring only | Per-socket buffer allocation is accepted by the chip. |
| `echo` | Cable + `sim.py` | First end-to-end datagram: receive 180 bytes, send 32 back. |
| `endian` | Cable + `sim.py` | Byte order, against an external reference `echo` cannot check. |
| `latency` | Cable + `sim.py` | Per-cycle microseconds — the figure the acceptance criterion asks for. |
| `soak` | Cable + `sim.py` | 500 Hz sustained for ≥10 minutes. **The acceptance test.** |

Every binary **repeats its report every ~5 seconds** instead of reporting once,
so a report is always still coming by the time you get a terminal open on the
serial port.

### Why `endian` exists separately from `echo`

`echo` is structurally unable to catch a byte-order bug. It decodes the inbound
payload and encodes its reply with the same code, so a firmware that gets the
byte order consistently wrong round-trips self-consistently and passes — both
ends of the "test" share the same mistake.

`endian` breaks that symmetry with an external reference: the host writes an
`f32` pattern the firmware never produced, and the firmware checks it against
constants compiled into itself. It checks two independent things:

- **The origin address**, decoded from the W5500's own receive header. This is
  where the driver's real byte-order exposure lives — registers are
  big-endian, and the driver never gets a chance to interpret payload bytes
  when reading them, so a swap here is a genuine driver bug.
- **The `f32` pattern and filler bytes** in the payload, which check that the
  little-endian payload arrived intact at the right offsets — a check `echo`
  cannot perform because it never compares against anything external.

### `soak`'s diagnostic reply

`soak` sends only every fifth received datagram (100 Hz replies against 500 Hz
inbound), so it cannot be a 1:1 echo. Its 32-byte reply is instead a
diagnostic frame, all little-endian:

| Bytes | Content |
|---|---|
| 0..4 | Firmware's last received sequence number (`u32`) |
| 4..8 | Firmware's total received-datagram count (`u32`) |
| 8..32 | Reserved, zero |

This is a deliberate departure from the production 8xf32 motor-command format
`echo`, `endian` and `latency` still send — `soak`'s job is to carry telemetry
for `host/sim.py`'s independent cross-check (below), not to exercise the
motor-command encoding path. That path stays covered by the other three
binaries.

### `soak`'s verdict needs both sides to agree

`soak`'s own counters reading zero is not, by itself, evidence of zero drops:
firmware that fails to notice a dropped datagram also has no way to report
one. The verdict requires **both**:

- the firmware's own counters (zero wrong-length datagrams, zero
  `skipped_sequences` — see `soak.rs`'s module doc for how that differs from
  `drained_extra`, the deliberately-discarded count from drain-to-latest), and
- `host/sim.py --mode soak` reporting `HOST-DECODED SEQUENCE GAPS: 0`, decoded
  independently from the diagnostic reply's sequence and received-count
  fields, with the firmware's reported received count matching the host's
  sent count.

Only agreement between the two independently-counting ends is meaningful. A
mid-run restart of `host/sim.py` is expected to log `peer restarted,
re-baselining` on the firmware side rather than a false FAIL.

## Host companion

`host/sim.py` stands in for the flight simulator. Python 3, standard library
only, and it is committed and runnable today:

```sh
python3 host/sim.py --mode echo                       # a few datagrams, prints replies
python3 host/sim.py --mode endian                     # the known f32 pattern
python3 host/sim.py --mode soak --rate 500 --seconds 600
```

In `soak` mode it decodes sequence gaps from the diagnostic reply itself,
independently of anything the firmware's own counters say, and it survives
socket errors rather than crashing. `echo` and `endian` reply 1:1, so
`sim.py` sends and blocks for a reply in lockstep there; `soak` replies only
every fifth received datagram, so lockstep would collapse the achieved send
rate to roughly 1/s — `soak` mode instead drives sends strictly at `--rate`
and drains whatever replies are available non-blockingly each iteration.

## Logging: USB serial, no probe required

Log output leaves over the RP2040's **own USB port** as a CDC-ACM serial device
(`log` + `embassy-usb-logger`). There is no debug probe on this board and no
`defmt`/RTT tooling involved — any serial terminal reads plain text. Baud rate
is irrelevant for USB CDC; pick anything.

The logger writes into a non-blocking pipe, so lines produced before a
terminal has opened the port are dropped rather than stalling the firmware.
That is why every binary loops and re-reports instead of running once.

## Building

```sh
rustup target add thumbv6m-none-eabi
cargo build --release
```

`.cargo/config.toml` sets the target, the linker scripts, and the `elf2uf2-rs`
runner; `memory.x` is the standard RP2040 layout and relies on embassy-rp's
default `BOOT_LOADER_W25Q080` second stage — a board with different flash
needs the matching `boot2-*` feature.

## Running

```sh
cargo install elf2uf2-rs --locked
```

Hold **BOOTSEL** while plugging the board in (or while tapping RESET) so it
mounts as the `RPI-RP2` drive, then `cargo run --release --bin <name>` builds,
converts to UF2, and copies it to the drive (`elf2uf2-rs -d`, configured as the
runner). The board reboots into the firmware and re-enumerates as a serial port
(`COMn` on Windows, `/dev/ttyACM0` on Linux) — open it in any terminal to watch
the report loop.

```sh
cargo run --release --bin identify   # expect: VERSIONR = 0x04, socket round trip OK
cargo run --release --bin spiclock   # expect: measured rate per candidate, integrity rounds clean
cargo run --release --bin link       # expect: link: UP, speed/duplex reported
cargo run --release --bin bufsize    # expect: Sn6 RX KB4 / TX KB2 accepted
cargo run --release --bin echo       # expect: datagrams received, 0 wrong length
cargo run --release --bin endian     # expect: endian: PASS, 0 byte-order faults
cargo run --release --bin latency    # expect: a worst-case cycle figure, comfortably under the 2 ms budget
cargo run --release --bin soak       # expect: ALL OK, 0 skipped, 0 wrong-length, and
                                      # host/sim.py --mode soak reporting
                                      # HOST-DECODED SEQUENCE GAPS: 0 (leave running >= 10 min)
```

None of this has run on real hardware yet — these are expected outcomes on
working hardware, not measured results. `latency` in particular exists to
produce the acceptance-criterion number; there is no figure to quote until it
has been run.

## Interpreting failures

| Symptom | Likely cause |
|---|---|
| `VERSIONR` reads `0x00`/`0xFF` | Data-line wiring (MISO/MOSI swapped or floating), CS, or power |
| `identify` passes, `spiclock` corrupts above some rate | Bus over-clocked for this board's trace lengths — back off to the last clean rate |
| `link: DOWN` | Cable, magnetics, or PHY power — nothing to do with this crate |
| `echo` times out while `link` is UP | IP/peer configuration mismatch, or `host/sim.py` is not running |
| `echo` passes but `endian` fails | A real byte-order bug in the driver or payload handling |
| `soak` reports drops while `latency` stays inside budget | Buffer sizing or a stall elsewhere in the cycle, not SPI throughput |
| USB serial port disappears | A panic halted the core — the failure mode these binaries route around by logging errors instead of unwrapping |

## Design notes

- `bufsize`/`soak` give socket 6 only 4 KiB of RX buffer, deliberately, not the
  chip's maximum. A deep buffer on a real-time rig is a liability: falling
  behind then means acting on *stale* datagrams that piled up rather than
  dropping a cycle and re-syncing to the present. 4 KiB is enough slack to
  absorb a scheduling hiccup while keeping any staleness bounded.
- The embassy crates version-lock each other — bump them together if
  resolution fails. `Cargo.lock` is committed so builds are reproducible.
- `embassy-usb-logger` is pinned to the 0.4 line on purpose: 0.6 needs
  `embassy-usb-driver` 0.2, and `embassy-rp` 0.4 provides 0.1.
