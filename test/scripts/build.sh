#!/usr/bin/env bash

set -eu

# Build distributable binaries for the test framework.
# TODO: Support macOS

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
TEST_FRAMEWORK_ROOT="$SCRIPT_DIR/.."
REPO_ROOT="$SCRIPT_DIR/../.."

# Build
build_linux() {
    mkdir -p "$TEST_FRAMEWORK_ROOT/dist"
    # Build the test manager
    "$SCRIPT_DIR/build/test-manager.sh" linux
    cp "$TEST_FRAMEWORK_ROOT/target/release/test-manager" "$TEST_FRAMEWORK_ROOT/dist/"

    # Build the test runner
    "$SCRIPT_DIR/build/test-runner.sh" linux
    cp "$TEST_FRAMEWORK_ROOT/target/x86_64-unknown-linux-gnu/release/test-runner" "$TEST_FRAMEWORK_ROOT/dist/"
    cp "$TEST_FRAMEWORK_ROOT/target/x86_64-unknown-linux-gnu/release/connection-checker" "$TEST_FRAMEWORK_ROOT/dist/"

    # Build mullvad-version
    cargo build --manifest-path="$REPO_ROOT/Cargo.toml" --release --bin mullvad-version
    cp "$REPO_ROOT/target/release/mullvad-version" "$TEST_FRAMEWORK_ROOT/dist/"
}

build_linux
