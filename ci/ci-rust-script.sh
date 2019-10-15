#!/usr/bin/env bash

set -eu

RUST_TOOLCHAIN_CHANNEL=$1
RUSTFLAGS="--deny unused_imports --deny dead_code"

source env.sh ""

RUST_EXTRA_COMPONENTS=""
if [ "${RUST_TOOLCHAIN_CHANNEL}" = "nightly" ]; then
  RUST_EXTRA_COMPONENTS+=" -c rustfmt-preview"
fi

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
  sh -s -- -y --default-toolchain $RUST_TOOLCHAIN_CHANNEL --profile minimal $RUST_EXTRA_COMPONENTS
source $HOME/.cargo/env

case "$(uname -s)" in
  MINGW*|MSYS_NT*)
    ./build_windows_modules.sh --dev-build
    ;;
esac

# FIXME: Becaues of our old jsonrpc dependency our Rust code won't build
# on latest nightly.
if [ "${RUST_TOOLCHAIN_CHANNEL}" != "nightly" ]; then
  cargo build --locked --verbose
  cargo test --locked --verbose
fi

if [ "${RUST_TOOLCHAIN_CHANNEL}" = "nightly" ]; then
  rustfmt --version;
  cargo fmt -- --check --unstable-features;
fi

if ! git diff-index --quiet HEAD; then
  echo "!!! Working directory is dirty !!!";
  git diff-index HEAD
  exit 1;
fi
