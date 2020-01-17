#!/usr/bin/env bash

set -eux

RUST_TOOLCHAIN_CHANNEL=$1
export RUSTFLAGS="--deny unused_imports --deny dead_code --deny unused_mut --deny unused_variables"

source env.sh ""

case "$(uname -s)" in
  Linux*|Darwin*)
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- \
      -y --default-toolchain none --profile minimal
    ;;
  MINGW*|MSYS_NT*)
    curl -sSf -o rustup-init.exe https://win.rustup.rs/
    ./rustup-init.exe -y --default-toolchain none --profile minimal --default-host x86_64-pc-windows-msvc
    # See https://github.com/rust-lang/rustup.rs/issues/2082
    RUST_TOOLCHAIN_CHANNEL="$RUST_TOOLCHAIN_CHANNEL-x86_64-pc-windows-msvc"
    ;;
esac
export PATH="$HOME/.cargo/bin/:$PATH"

# Install the toolchain together with rustfmt. Here -c backtracks to last version where
# the component was available.
time rustup toolchain install $RUST_TOOLCHAIN_CHANNEL --no-self-update -c rustfmt

case "$(uname -s)" in
  MINGW*|MSYS_NT*)
    export PATH="/c/Program Files (x86)/Microsoft Visual Studio/2019/BuildTools/MSBuild/Current/Bin/amd64/:$PATH"
    time ./build_windows_modules.sh --dev-build
    ;;
esac

# Build wireguard-go
# On Windows, it relies on having msbuild.exe in your path.
bash build-wireguard-go.sh

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
