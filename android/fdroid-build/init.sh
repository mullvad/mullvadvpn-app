#!/usr/bin/env bash

set -eux

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
REPO_DIR="$( cd "$SCRIPT_DIR/../.." && pwd )"

BUILD_DOCKERFILE="$REPO_DIR/building/Dockerfile"
RUSTUP_VERSION="$(sed -n 's/^ARG RUSTUP_VERSION=\(.*\)$/\1/p' "$BUILD_DOCKERFILE")"
RUSTUP_SHA256_CHECKSUM="$(sed -n 's/^ARG RUSTUP_INIT_SHA256=\(.*\)$/\1/p' "$BUILD_DOCKERFILE")"

# Install Rust
curl -sf -L https://static.rust-lang.org/rustup/archive/"$RUSTUP_VERSION"/x86_64-unknown-linux-gnu/rustup-init > /tmp/rustup-init
echo "$RUSTUP_SHA256_CHECKSUM /tmp/rustup-init" | sha256sum -c
chmod +x /tmp/rustup-init
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
