#!/usr/bin/env bash

# Run the container to generate node gRPC bindings from .proto files.
#
# Requires the container to have been built first, please refer to the documentation
# for more info on how to use this script.

set -eu

CONTAINER_RUNNER=${CONTAINER_RUNNER:-"podman"}
IMAGE_HASH="4c6c9f0924"
IMAGE_NAME="ghcr.io/mullvad/mullvadvpn-app-build-node-grpc-bindings:$IMAGE_HASH"

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROTO_DIR="$( cd "$SCRIPT_DIR/../../../../mullvad-management-interface/proto" && pwd )"
OUT_DIR="$SCRIPT_DIR/../dist"

set -x
exec "$CONTAINER_RUNNER" run --rm -it \
    -v "$PROTO_DIR:/proto:Z" \
    -v "$OUT_DIR:/proto-bindings:Z" \
    "$IMAGE_NAME"
