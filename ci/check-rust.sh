#!/usr/bin/env bash

set -eux

export RUSTFLAGS="--deny warnings"

# Build WireGuard Go
./wireguard/build-wireguard-go.sh

# Build Windows modules
case "$(uname -s)" in
  MINGW*|MSYS_NT*)
    time ./build_windows_modules.sh --dev-build
    ;;
esac

# Build Rust crates
source env.sh
time cargo build --locked --verbose

# Test Rust crates
time cargo test --locked --verbose
