#!/usr/bin/env bash

# Runs cargo with the path remapping argument added.
#
# Exists for the npm scripts building the native node modules, which cannot
# obtain the argument themselves since npm runs them through cmd.exe on
# Windows, where command substitution is unavailable.
#
# Usage: building/cargo-with-path-remapping.sh build --release --target <triple>

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# Assigned first because `set -e` does not catch a failing command substitution
# used directly as an argument.
remap_path_prefix_arg=$("$SCRIPT_DIR/rustc-remap-path-prefix.sh")

exec cargo "$@" "$remap_path_prefix_arg"
