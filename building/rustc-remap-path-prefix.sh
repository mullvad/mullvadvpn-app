#!/usr/bin/env bash

# Returns the rustc `--remap-path-prefix` flags needed to replace file paths
# that gets put in the build artifacts with fixed values in order to make
# the build reproducible across different machines.

set -eu

SOURCE_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && cd .. && pwd )"
CARGO_HOME_PATH=${CARGO_HOME:-$HOME/.cargo}
RUSTUP_HOME_PATH=${RUSTUP_HOME:-$HOME/.rustup}

echo "--remap-path-prefix $CARGO_HOME_PATH=/CARGO_HOME \
--remap-path-prefix $RUSTUP_HOME_PATH=/RUSTUP_HOME \
--remap-path-prefix $SOURCE_DIR=/SOURCE_DIR"
