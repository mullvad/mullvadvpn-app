#!/usr/bin/env bash

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR/.."

TARGET=${1:-$(rustc -vV | sed -n 's|host: ||p')}
PRODUCT_VERSION=$(cargo run -q --bin mullvad-version)

ASSETS=(
    "build-standalone/src/renderer/lib/routes.js"
    "build-standalone/src/renderer/lib/foundations/*.js"
    "build-standalone/src/renderer/lib/foundations/**/*.js"
    "build-standalone/src/shared/constants/*.js"
    "build-standalone/test/e2e/utils.js"
    "build-standalone/test/e2e/shared/*.js"
    "build-standalone/test/e2e/installed/*.js"
    "build-standalone/test/e2e/installed/**/*.js"
)

NODE_MODULES=(
    "node_modules/.bin/playwright"
    "node_modules/playwright"
    "node_modules/playwright-core"
    "node_modules/@playwright/test"
)

function build_test_executable {
    local pkg_target=$1
    local bin_suffix=${2:-""}
    local temp_dir
    temp_dir="$(mktemp -d)"
    local temp_executable="$temp_dir/temp-test-executable$bin_suffix"
    local output_name="app-e2e-tests-$PRODUCT_VERSION-$TARGET$bin_suffix"
    local output="../../../dist/$output_name"
    local node_copy_path="$temp_dir/node$bin_suffix"
    local node_path
    node_path="$(volta which node || which node)"

    # pack assets
    cp "$node_path" "$node_copy_path"
    # shellcheck disable=SC2068
    tar -czf ./build-standalone/assets.tar.gz ${ASSETS[@]} -C ../../ ${NODE_MODULES[@]}

    cp "$node_copy_path" "$temp_executable"
    node --experimental-sea-config standalone-tests.sea.json

    # Inject SEA blob
    case $pkg_target in
        macos-*)
            codesign --remove-signature "$temp_executable"
            npx postject "$temp_executable" NODE_SEA_BLOB \
                standalone-tests.sea.blob --sentinel-fuse NODE_SEA_FUSE_fce680ab2cc467b6e072b8b5df1996b2 \
                --macho-segment-name NODE_SEA
            codesign --sign - "$temp_executable"
            ;;
        *)
            npx postject "$temp_executable" NODE_SEA_BLOB \
                standalone-tests.sea.blob --sentinel-fuse NODE_SEA_FUSE_fce680ab2cc467b6e072b8b5df1996b2
            ;;
    esac

    mkdir -p "$(dirname "$output")"
    mv "$temp_executable" "$output"
    echo "Test executable created: $output_name"

    rm -rf "$temp_dir"
}

npm run build:standalone

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
