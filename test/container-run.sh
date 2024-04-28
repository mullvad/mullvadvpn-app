#!/usr/bin/env bash

set -eu

CARGO_REGISTRY_VOLUME_NAME=${CARGO_REGISTRY_VOLUME_NAME:-"cargo-registry"}
CONTAINER_RUNNER=${CONTAINER_RUNNER:-"podman"}

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
REPO_DIR="$SCRIPT_DIR/.."
cd "$SCRIPT_DIR"

source "$REPO_DIR/scripts/utils/log"

if [[ ${1:-""} != "linux" ]]; then
    log_error "Invalid platform. Specify a valid platform as first argument"
    exit 1
fi

shift

if ! "$CONTAINER_RUNNER" image exists mullvadvpn-app-tests; then
    container_image=$(cat "$REPO_DIR/building/linux-container-image.txt")
    podman build -t mullvadvpn-app-tests --build-arg IMAGE="${container_image}" .
fi

set -x
exec "$CONTAINER_RUNNER" run --rm -it \
    -v "${CARGO_REGISTRY_VOLUME_NAME}":/root/.cargo/registry:Z \
    -v "${REPO_DIR}":/build:z \
    -w "/build/test" \
    -e CARGO_TARGET_DIR=/build/test/target \
    mullvadvpn-app-tests \
    /bin/bash -c "$*"
