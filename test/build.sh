#!/usr/bin/env bash

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
APP_DIR="$SCRIPT_DIR/.."
cd "$SCRIPT_DIR"

if [[ $TARGET == x86_64-unknown-linux-gnu ]]; then
    mkdir -p .container/cargo-registry
    container_image=$(cat "$APP_DIR/building/linux-container-image.txt")

    podman run --rm -it \
        -v "${SCRIPT_DIR}/.container/cargo-registry":/root/.cargo/registry \
        -v "${APP_DIR}":/src:Z \
        -e CARGO_HOME=/root/.cargo/registry \
        "${container_image}" \
        /bin/bash -c "cd /src/test/; cargo build --bin test-runner --release --target ${TARGET}"
else
    cargo build --bin test-runner --release --target "${TARGET}"
fi

# Don't build a runner image for macOS.
if [[ $TARGET != aarch64-apple-darwin ]]; then
    ./scripts/build-runner-image.sh
fi
