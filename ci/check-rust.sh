#!/usr/bin/env bash

set -eux

export RUSTFLAGS="--deny warnings"

# Build WireGuard Go
if [[ "$(uname -s)" != "MINGW"* ]]; then
    ./wireguard/build-wireguard-go.sh
fi

# Build Rust crates
source env.sh
time cargo build --locked --verbose

# Test Rust crates
time cargo test --locked --verbose
