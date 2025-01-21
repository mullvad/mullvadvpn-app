#!/usr/bin/env bash

set -eux

export RUSTFLAGS="--deny warnings"

# Excluding windows-installer because it builds an actual full installer
# at the build step, which is not desired in a CI environment.

# Build Rust crates
source env.sh
time cargo build --workspace --exclude windows-installer --locked --verbose

# Test Rust crates
time cargo test --workspace --exclude windows-installer --locked --verbose
