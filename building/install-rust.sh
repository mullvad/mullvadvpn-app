#!/usr/bin/env bash

set -eu

# Pin and verify rustup version. When updating, get the checksum from ${RUSTUP_INIT_URL}.sha256
RUSTUP_VERSION=1.29.1
RUSTUP_INIT_SHA256=dda7234360b7f578ca8b0ddcb80145646fa61a67c1720a5abc7051b35c9fcb71
RUSTUP_INIT_URL=https://static.rust-lang.org/rustup/archive/$RUSTUP_VERSION/x86_64-unknown-linux-gnu/rustup-init

case ${1-} in
    linux)
        targets=(aarch64-unknown-linux-gnu x86_64-pc-windows-gnu)
    ;;
    android)
        targets=(x86_64-linux-android i686-linux-android aarch64-linux-android armv7-linux-androideabi)
    ;;
    *)
        echo "Usage: $0 <linux|android>" >&2
        exit 1
esac

curl --proto '=https' --tlsv1.2 -sSf -o /tmp/rustup-init "$RUSTUP_INIT_URL"
echo "$RUSTUP_INIT_SHA256  /tmp/rustup-init" | sha256sum -c -
chmod +x /tmp/rustup-init
/tmp/rustup-init -y --default-toolchain none --profile minimal
rm /tmp/rustup-init

PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH"
cd "$(dirname "$0")"
rustup toolchain install --no-self-update
rustup component add clippy
rustup target add "${targets[@]}"
rustup default "$(rustup show active-toolchain | cut -d' ' -f1)"
