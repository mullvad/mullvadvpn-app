#!/usr/bin/env bash

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"
cd ..

TARGET=${1:-""}
PRODUCT_VERSION=$(cargo run -q --bin mullvad-version)

function build_test_executable {
    local target=$1
    local suffix=${2:-""}
    local output="../dist/app-e2e-tests-$PRODUCT_VERSION-$target$suffix"

    npm exec pkg -- \
        --config test.pkg.json \
        --targets "$target" \
        --output "$output" \
        build/standalone-tests.js
}

if [[ -n ${TARGET:-""} ]]; then
    case "$TARGET" in
    "aarch64-unknown-linux-gnu"*)
        build_test_executable linux-arm64
        ;;
    "x86_64-unknown-linux-gnu"*)
        build_test_executable linux-x64
        ;;
    "aarch64-apple-darwin"*)
        build_test_executable macos-arm64
        ;;
    "x86_64-apple-darwin"*)
        build_test_executable macos-x64
        ;;
    "x86_64-pc-windows-mscv"*)
        build_test_executable win-x64 .exe
        ;;
    esac
else
    case "$(uname -s)" in
    MINGW*|MSYS_NT*)
        build_test_executable win-x64 .exe
        ;;
    Linux*)
        if [[ "$(uname -m)" == "x86_64" ]]; then
            build_test_executable linux-x64
        else
            build_test_executable linux-arm64
        fi
        ;;
    Darwin*)
        if [[ "$(uname -m)" == "x86_64" ]]; then
            build_test_executable macos-x64
        else
            build_test_executable macos-arm64
        fi
        ;;
    esac
fi
