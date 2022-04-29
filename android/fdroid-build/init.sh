#!/usr/bin/env bash

set -eux

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

REPO_DIR="$SCRIPT_DIR/../../"
TOOLCHAINS_DIR="$HOME/android-ndk-toolchains"

# Install Rust
curl -sf -L https://sh.rustup.rs > /tmp/rustup.sh
chmod +x /tmp/rustup.sh
/tmp/rustup.sh -y
source "$HOME/.cargo/env"
rustup set profile minimal
rustup target add \
    i686-linux-android \
    x86_64-linux-android \
    aarch64-linux-android \
    armv7-linux-androideabi

# Install Go
cd "$HOME"
curl -sf -L -O https://go.dev/dl/go1.16.linux-amd64.tar.gz
echo "013a489ebb3e24ef3d915abe5b94c3286c070dfe0818d5bca8108f1d6e8440d2 go1.16.linux-amd64.tar.gz" | sha256sum -c
tar -xzvf go1.16.linux-amd64.tar.gz
patch -p1 -f -N -r- -d "$HOME/go" < "$REPO_DIR/wireguard/libwg/goruntime-boottime-over-monotonic.diff"

# Configure Cargo for cross-compilation
sed -e "s|{NDK_PATH}|$NDK_PATH|g" "$SCRIPT_DIR/cargo-config.toml.template" > "$HOME/.cargo/config"
