#!/usr/bin/env bash

# Runs clippy on all cargo targets for a single target triple, both with all
# features off and on. `cargo clippy` only type checks, so it needs the standard
# library for that triple, but neither a linker nor a test runner for that
# platform.
#
# The triple is always required, even for the host, so that what gets linted is
# stated rather than implied by whatever machine runs this. Which packages get
# linted follows from the triple and from the workspace in the current
# directory, so that every caller is just this script and a triple.
#
# Usage: $ cargo-ci-clippy.sh <target triple>

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

if [[ $# -ne 1 ]]; then
    echo "Usage: $0 <target triple>" >&2
    exit 1
fi
target=$1

# Tell the workspaces apart by a member only the test framework has.
# TODO: Read the member list with jq once the build containers ship it.
members="$(cargo metadata --no-deps --locked --format-version 1)"
case $members in
    *'"name":"test-manager"'*) workspace=test-framework ;;
    *) workspace=app ;;
esac

case $workspace in
    app)
        # The mobile platforms only build a few of our crates.
        case $target in
            *-apple-ios*) packages=(-p mullvad-ios -p mullvad-api) ;;
            *-android*) packages=(-p mullvad-jni) ;;
            *) packages=(--workspace) ;;
        esac
        ;;
    test-framework)
        packages=(--workspace)
        # test-manager is not built for Windows.
        case $target in
            *-windows-*) packages+=(--exclude test-manager) ;;
        esac
        ;;
esac

# Lint both with all features off and on, to cover code that only compiles with
# a feature enabled as well as the code that feature excludes.
for features in --no-default-features --all-features; do
    "$SCRIPT_DIR/cargo-ci.sh" clippy --all-targets "$features" "${packages[@]}" \
        --target "$target"
done
