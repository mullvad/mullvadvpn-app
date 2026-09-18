#!/usr/bin/env bash

# Runs clippy, with all features and on all targets, for a single compilation
# target. `cargo clippy` only type checks, so the target needs its standard
# library, but neither a linker nor a runner of that platform.
#
# The target is always required, even for the host, so that what gets linted
# is stated rather than implied by whatever machine runs this. Operates on the
# workspace in the current directory.
#
# Usage: $ cargo-ci-clippy.sh <target triple>

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

if [[ $# -ne 1 ]]; then
    echo "Usage: $0 <target triple>" >&2
    exit 1
fi
target=$1

# The mobile platforms only build a few of our crates.
case $target in
    *-apple-ios*) packages=(-p mullvad-ios -p mullvad-api) ;;
    *-android*) packages=(-p mullvad-jni) ;;
    *) packages=(--workspace) ;;
esac

exec "$SCRIPT_DIR/cargo-ci.sh" clippy --all-targets --all-features "${packages[@]}" \
    --target "$target"
