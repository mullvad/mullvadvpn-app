set -eu
RUST_TOOLCHAIN_CHANNEL=$1
RUSTFLAGS="--deny unused_imports --deny dead_code"

source env.sh ""
rustup update $RUST_TOOLCHAIN_CHANNEL
rustup default $RUST_TOOLCHAIN_CHANNEL

cargo build --verbose
cargo test --verbose
if [ "${RUST_TOOLCHAIN_CHANNEL}" = "nightly" ]; then
  rustup component add rustfmt-preview;
  rustfmt --version;
  cargo fmt -- --check --unstable-features;
fi
