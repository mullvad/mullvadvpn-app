#!/usr/bin/env bash

# Returns the rustc `--remap-path-prefix` flags needed to replace file paths
# that gets put in the build artifacts with fixed values in order to make
# the build reproducible across different machines.

set -eu

CARGO_HOME_PATH=${CARGO_HOME:-$HOME/.cargo}
RUSTUP_HOME_PATH=${RUSTUP_HOME:-$HOME/.rustup}
SOURCE_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && cd .. && pwd )"
CARGO_TARGET_DIR_PATH=${CARGO_TARGET_DIR:-$SOURCE_DIR/target}

# The order is significant. Iterated over in reverse, so last argument is
# treated as most significant. Since CARGO_TARGET_DIR_PATH might be located
# under $SOURCE_DIR, it's important to keep it last.
# See: https://github.com/rust-lang/rust/blob/55459598c250d985eb5f840306dfb59f267c03b6/compiler/rustc_span/src/source_map.rs#L1125-L1127
echo "\
--remap-path-prefix $CARGO_HOME_PATH=/CARGO_HOME \
--remap-path-prefix $RUSTUP_HOME_PATH=/RUSTUP_HOME \
--remap-path-prefix $SOURCE_DIR=/SOURCE_DIR \
--remap-path-prefix $CARGO_TARGET_DIR_PATH=/CARGO_TARGET_DIR \
" | xargs
