#!/usr/bin/env bash

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR/.."

TARGET=${1:-$(rustc -vV | sed -n 's|host: ||p')}
PRODUCT_VERSION=$(cargo run -q --bin mullvad-version)

ASSETS=(
    "build/src/config.json"
    "build/src/renderer/lib/routes.js"
    "build/test/e2e/utils.js"
    "build/test/e2e/shared/*.js"
    "build/test/e2e/installed/*.js"
    "build/test/e2e/installed/**/*.js"
    "node_modules/.bin/playwright"
    "node_modules/playwright"
    "node_modules/playwright-core"
    "node_modules/@playwright/test"
)

function build_test_executable {
    local pkg_target=$1
    local suffix=${2:-""}
    local temp_output="./build/temp-test-executable"
    local output="../dist/app-e2e-tests-$PRODUCT_VERSION-$TARGET$suffix"

    # pack assets
    cp "$(volta which node)" ./build/test/node
    # shellcheck disable=SC2068
    tar -czf ./build/test/assets.tar.gz ${ASSETS[@]}

    cp ./build/test/node "$temp_output"
    node --experimental-sea-config standalone-tests.sea.json

    # Inject SEA blob
    case $pkg_target in
        macos-*)
            codesign --remove-signature "$temp_output"
            npx postject "$temp_output" NODE_SEA_BLOB \
                standalone-tests.sea.blob --sentinel-fuse NODE_SEA_FUSE_fce680ab2cc467b6e072b8b5df1996b2 \
                --macho-segment-name NODE_SEA
            codesign --sign - "$temp_output"
            ;;
        *)
            npx postject "$temp_output" NODE_SEA_BLOB \
                standalone-tests.sea.blob --sentinel-fuse NODE_SEA_FUSE_fce680ab2cc467b6e072b8b5df1996b2
            ;;
    esac

    mkdir -p "$(dirname "$output")"
    mv "$temp_output" "$output"
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
