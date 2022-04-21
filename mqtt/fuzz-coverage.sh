#!/usr/bin/env bash
set -euo pipefail

cargo +nightly fuzz coverage process
cargo +nightly cov -- show \
    fuzz/target/x86_64-unknown-linux-gnu/release/process \
    --format=html \
    -instr-profile=fuzz/coverage/process/coverage.profdata \
    > index.html
