#!/usr/bin/env bash

set -eux

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

REPO_DIR="$SCRIPT_DIR/../../"

# Install Rust
curl -sf -L https://sh.rustup.rs > /tmp/rustup.sh
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

# Install golang
GOLANG_VERSION="1.21.3"
# Checksum from: https://golang.org/dl/
GOLANG_HASH="1241381b2843fae5a9707eec1f8fb2ef94d827990582c7c7c32f5bdfbfd420c8"
cd "$HOME"
curl -sf -L -o go.tgz https://go.dev/dl/go${GOLANG_VERSION}.linux-amd64.tar.gz
echo "$GOLANG_HASH go.tgz" | sha256sum -c
tar -xzvf go.tgz
patch -p1 -f -N -r- -d "$HOME/go" < "$REPO_DIR/wireguard/libwg/goruntime-boottime-over-monotonic.diff"

# Configure Cargo for cross-compilation
sed -e "s|{NDK_PATH}|$NDK_PATH|g" "$SCRIPT_DIR/cargo-config.toml.template" > "$HOME/.cargo/config"
