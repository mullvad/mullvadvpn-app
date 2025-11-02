#!/usr/bin/env bash

# Script for building a statically linked Mullvad VPN app.
#
# # Prerequisites
# - zig (https://ziglang.org/download/)
# - cargo-zigbuild (https://github.com/rust-cross/cargo-zigbuild)

set -eux

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# Install the appropriate rust target(s)
rustup target add armv7-unknown-linux-musleabihf

# Build the app fully statically linked! :-)
RUSTFLAGS="-C target-feature=+crt-static" cargo zigbuild --target armv7-unknown-linux-musleabihf "$@"
