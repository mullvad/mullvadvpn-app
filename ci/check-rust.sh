#!/usr/bin/env bash

set -eux

export RUSTFLAGS="--deny warnings"

# Build Rust crates
source env.sh
time cargo build --locked --verbose

# Test Rust crates
time cargo test --locked --verbose
