#!/usr/bin/env bash

set -eu

CARGO_REGISTRY_VOLUME_NAME=${CARGO_REGISTRY_VOLUME_NAME:-"cargo-registry"}
CONTAINER_RUNNER=${CONTAINER_RUNNER:-"podman"}
PACKAGE_FOLDER=${PACKAGE_FOLDER:-"$HOME/.cache/mullvad-test/packages"}

if [ ! -d "$PACKAGE_FOLDER" ]; then
  echo "$PACKAGE_FOLDER does not exist. It is needed to build the test bundle, so please go ahead and create the directory and re-run this script."
fi

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
REPO_DIR="$SCRIPT_DIR/../.."
cd "$SCRIPT_DIR"

source "$REPO_DIR/scripts/utils/log"

if [[ "$(uname -s)" != "Linux" ]]; then
    log_error "$0 only works on Linux"
    exit 1
fi

container_image=$(cat "$REPO_DIR/building/linux-container-image.txt")
podman build -t mullvadvpn-app-tests --build-arg IMAGE="${container_image}" .

exec "$CONTAINER_RUNNER" run --rm -it \
    -v "${CARGO_REGISTRY_VOLUME_NAME}":/root/.cargo/registry:Z \
    -v "${REPO_DIR}":/build:z \
    -w "/build/test" \
    -e CARGO_TARGET_DIR=/build/test/target \
    -v "${PACKAGE_FOLDER}":/packages:Z \
    -e PACKAGE_FOLDER=/packages \
    mullvadvpn-app-tests \
    /bin/bash -c "$*"
