#!/usr/bin/env bash

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR/.."

TARGET=${1:-$(rustc -vV | sed -n 's|host: ||p')}
PRODUCT_VERSION=$(cargo run -q --bin mullvad-version)

function build_test_executable {
    local pkg_target=$1
    local suffix=${2:-""}
    local output="../dist/app-e2e-tests-$PRODUCT_VERSION-$TARGET$suffix"

    npm exec pkg -- \
        --config standalone-tests.pkg.json \
        --targets "$pkg_target" \
        --output "$output" \
        build/standalone-tests.js
}

case "$TARGET" in
    "aarch64-unknown-linux-gnu")
        build_test_executable linux-arm64
        ;;
    "x86_64-unknown-linux-gnu")
        build_test_executable linux-x64
        ;;
    "aarch64-apple-darwin")
        build_test_executable macos-arm64
        ;;
    "x86_64-apple-darwin")
        build_test_executable macos-x64
        ;;
    "x86_64-pc-windows-msvc")
        build_test_executable win-x64 .exe
        ;;
esac
