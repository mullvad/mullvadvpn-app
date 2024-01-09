#!/usr/bin/env bash

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
APP_DIR="$SCRIPT_DIR/.."
cd "$SCRIPT_DIR"

if [[ $TARGET == x86_64-unknown-linux-gnu ]]; then
    mkdir -p .container/cargo-registry
    container_image=$(cat "$APP_DIR/building/linux-container-image.txt")
    podman build -t mullvadvpn-app-tests --build-arg IMAGE="${container_image}" .

    podman run --rm -it \
        -v "${SCRIPT_DIR}/.container/cargo-registry":/root/.cargo/registry \
        -v "${APP_DIR}":/src:Z \
        -e CARGO_HOME=/root/.cargo/registry \
        -e CARGO_TARGET_DIR=/src/test/target \
        mullvadvpn-app-tests \
        /bin/bash -c "cd /src/test/; cargo build --bin test-runner --release --target ${TARGET}"
else
    cargo build --bin test-runner --release --target "${TARGET}"
fi

# Only build runner image for Windows
if [[ $TARGET == x86_64-pc-windows-gnu ]]; then
    ./scripts/build-runner-image.sh
fi
