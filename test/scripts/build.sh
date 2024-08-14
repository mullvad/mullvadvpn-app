#!/usr/bin/env bash

set -eu

# Build distributable binaries for the test framework.
# TODO: Support macOS

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
TEST_FRAMEWORK_ROOT="$SCRIPT_DIR/.."

# Build
build_linux() {
    # Build the test manager
    "$SCRIPT_DIR/container-run.sh" bash -c "cd $TEST_FRAMEWORK_ROOT; OPENSSL_STATIC=1 OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu OPENSSL_INCLUDE_DIR=/usr/include/openssl cargo build -p test-manager --release"
    mkdir -p "$TEST_FRAMEWORK_ROOT/dist"
    cp "$TEST_FRAMEWORK_ROOT/target/release/test-manager" "$TEST_FRAMEWORK_ROOT/dist/"

    # Build the test runner
    "$SCRIPT_DIR/build-runner.sh" linux
    cp "$TEST_FRAMEWORK_ROOT/target/x86_64-unknown-linux-gnu/release/test-runner" "$TEST_FRAMEWORK_ROOT/dist/"
    cp "$TEST_FRAMEWORK_ROOT/target/x86_64-unknown-linux-gnu/release/connection-checker" "$TEST_FRAMEWORK_ROOT/dist/"
}

build_linux
