#!/usr/bin/env bash

set -eux

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

RUSTUP_VERSION="1.29.1"
RUSTUP_SHA256_CHECKSUM="dda7234360b7f578ca8b0ddcb80145646fa61a67c1720a5abc7051b35c9fcb71"

# Install Rust
curl -sf -L https://static.rust-lang.org/rustup/archive/$RUSTUP_VERSION/x86_64-unknown-linux-gnu/rustup-init > /tmp/rustup-init
echo "$RUSTUP_SHA256_CHECKSUM /tmp/rustup-init" | sha256sum -c
/tmp/rustup-init -y
# shellcheck source=/dev/null
source "$HOME/.cargo/env"
rustup set profile minimal
rustup target add \
    i686-linux-android \
    x86_64-linux-android \
    aarch64-linux-android \
    armv7-linux-androideabi

# Configure Cargo for cross-compilation
sed -e "s|{NDK_PATH}|$NDK_PATH|g" "$SCRIPT_DIR/cargo-config.toml.template" > "$HOME/.cargo/config"
