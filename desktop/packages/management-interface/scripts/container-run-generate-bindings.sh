#!/usr/bin/env bash

# Run the management-interface container and generate JS bindings from proto files.
#
# Requires the container to have been built first, please refer to the documentation
# for more info on how to use this script.

set -eu

CONTAINER_RUNNER=${CONTAINER_RUNNER:-"podman"}

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROTO_DIR="$( cd "$SCRIPT_DIR/../../../../mullvad-management-interface/proto" && pwd )"
IMAGE_NAME="grpc-js-ts-bindings"
OUT_DIR="$SCRIPT_DIR/../dist"
MOUNT_TARGET_BASE="/build"

cd "$SCRIPT_DIR"

mkdir -p "$OUT_DIR"

# TODO: Verify that container exists or exit with error message
exec "$CONTAINER_RUNNER" run --rm -it \
    -v "$PROTO_DIR:$MOUNT_TARGET_BASE/proto:Z" \
    -v "$OUT_DIR:$MOUNT_TARGET_BASE/dist:Z" \
    "$IMAGE_NAME" bash -c ". ~/.bashrc && /build/generate-bindings.sh"
