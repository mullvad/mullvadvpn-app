#!/usr/bin/env bash

set -eux

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

RUSTUP_SHA256_CHECKSUM="7d0ea0f8eba7fa1ebfe998091cd7ec4501e33ec5ca6b884eb4d894d7da5170af"

# Install Rust
curl -sf -L https://sh.rustup.rs > /tmp/rustup.sh
echo "$RUSTUP_SHA256_CHECKSUM /tmp/rustup.sh" | sha256sum -c
chmod +x /tmp/rustup.sh
/tmp/rustup.sh -y
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
