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

# Install golang
GOLANG_VERSION="1.18.5"
# Checksum from: https://golang.org/dl/
GOLANG_HASH="9e5de37f9c49942c601b191ac5fba404b868bfc21d446d6960acc12283d6e5f2"
cd "$HOME"
curl -sf -L -o go.tgz https://go.dev/dl/go${GOLANG_VERSION}.linux-amd64.tar.gz
echo "$GOLANG_HASH go.tgz" | sha256sum -c
tar -xzvf go.tgz
patch -p1 -f -N -r- -d "$HOME/go" < "$REPO_DIR/wireguard/libwg/goruntime-boottime-over-monotonic.diff"

# Configure Cargo for cross-compilation
sed -e "s|{NDK_PATH}|$NDK_PATH|g" "$SCRIPT_DIR/cargo-config.toml.template" > "$HOME/.cargo/config"
