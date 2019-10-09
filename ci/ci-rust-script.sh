#!/usr/bin/env bash

set -eu

RUST_TOOLCHAIN_CHANNEL=$1
RUSTFLAGS="--deny unused_imports --deny dead_code"

source env.sh ""

rustup update $RUST_TOOLCHAIN_CHANNEL
rustup default $RUST_TOOLCHAIN_CHANNEL

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
  if rustup component add rustfmt-preview; then
    rustfmt --version;
    cargo fmt -- --check --unstable-features;
  else
    echo "There seems to not be any rustfmt for the current nighly. Skipping formatting check!"
  fi
fi

if ! git diff-index --quiet HEAD; then
  echo "!!! Working directory is dirty !!!";
  git diff-index HEAD
  exit 1;
fi
