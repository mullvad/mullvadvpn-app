#!/usr/bin/env bash

set -x

rustup target add x86_64-unknown-linux-musl

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
export LIBMNL_LIB_DIR="$SCRIPT_DIR/../dist-assets/binaries/x86_64-unknown-linux-musl"
export LIBNFTNL_LIB_DIR="$SCRIPT_DIR/../dist-assets/binaries/x86_64-unknown-linux-musl"

RUSTFLAGS="-C target-feature=+crt-static" cargo b --target x86_64-unknown-linux-musl --features boringtun
