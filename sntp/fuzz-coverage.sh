#!/usr/bin/env bash
set -euo pipefail

cargo +nightly fuzz coverage on_recv_interrupt
cargo +nightly cov -- show \
    fuzz/target/x86_64-unknown-linux-gnu/release/on_recv_interrupt \
    --format=html \
    -instr-profile=fuzz/coverage/on_recv_interrupt/coverage.profdata \
    > index.html
