#!/usr/bin/env bash
set -euo pipefail

cargo +nightly fuzz coverage response
cargo +nightly cov -- show \
    fuzz/target/x86_64-unknown-linux-gnu/release/response \
    --format=html \
    -instr-profile=fuzz/coverage/response/coverage.profdata \
    > index.html
