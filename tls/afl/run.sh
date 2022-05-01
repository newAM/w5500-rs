#!/usr/bin/env bash
set -euo pipefail

rm -r 'out' || true
rm -r 'in' || true
cargo run -p w5500-tls-afl --bin gencorpus
mkdir 'in'
mv corpus 'in'
cargo afl build -p w5500-tls-afl
cargo afl fuzz -i in -o out target/debug/w5500-tls-afl
