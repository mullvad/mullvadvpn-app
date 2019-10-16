#!/usr/bin/env bash

set -eu

RUST_TOOLCHAIN_CHANNEL=$1
RUSTFLAGS="--deny unused_imports --deny dead_code"

source env.sh ""

RUST_EXTRA_COMPONENTS=""
if [ "${RUST_TOOLCHAIN_CHANNEL}" = "nightly" ]; then
  RUST_EXTRA_COMPONENTS+=" -c rustfmt-preview"
fi

# Install Rust if on Linux or macOS
if [[ "$(uname -s)" == "Linux" || "$(uname -s)" == "Darwin" ]]; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- \
    -y --default-toolchain $RUST_TOOLCHAIN_CHANNEL --profile minimal $RUST_EXTRA_COMPONENTS
  source $HOME/.cargo/env
fi

case "$(uname -s)" in
  MINGW*|MSYS_NT*)
    time ./build_windows_modules.sh --dev-build
    ;;
esac

# FIXME: Becaues of our old jsonrpc dependency our Rust code won't build
# on latest nightly.
if [ "${RUST_TOOLCHAIN_CHANNEL}" != "nightly" ]; then
  time cargo build --locked --verbose
  time cargo test --locked --verbose
fi

if [[ "${RUST_TOOLCHAIN_CHANNEL}" == "nightly" && "$(uname -s)" == "Linux" ]]; then
  rustfmt --version;
  cargo fmt -- --check --unstable-features;
fi

if ! git diff-index --quiet HEAD; then
  echo "!!! Working directory is dirty !!!";
  git diff-index HEAD
  exit 1;
fi
