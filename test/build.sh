#!/usr/bin/env bash

set -eu

if [[ -z ${TARGET:-""} ]]; then
    echo "\$TARGET must be specified"
    exit 1
fi

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
APP_DIR="$SCRIPT_DIR/.."
cd "$SCRIPT_DIR"

if [[ $TARGET == x86_64-unknown-linux-gnu ]]; then
    CARGO_REGISTRY_VOLUME_NAME=${CARGO_REGISTRY_VOLUME_NAME:-"cargo-registry"}

    if ! podman image exists mullvadvpn-app-tests; then
        container_image=$(cat "$APP_DIR/building/linux-container-image.txt")
        podman build -t mullvadvpn-app-tests --build-arg IMAGE="${container_image}" .
    fi

    podman run --rm -it \
        -v "${CARGO_REGISTRY_VOLUME_NAME}":/root/.cargo/registry:Z \
        -v "${APP_DIR}":/src:z \
        -e CARGO_TARGET_DIR=/src/test/target \
        mullvadvpn-app-tests \
        /bin/bash -c "cd /src/test/; cargo build --bin test-runner --bin connection-checker --release --target ${TARGET}"
else
    cargo build \
        --bin test-runner \
        --bin connection-checker \
        --release --target "${TARGET}"
fi

# Only build runner image for Windows
if [[ $TARGET == x86_64-pc-windows-gnu ]]; then
    ./scripts/build-runner-image.sh
fi
