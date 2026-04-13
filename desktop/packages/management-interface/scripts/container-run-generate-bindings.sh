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
OUT_DIR="$SCRIPT_DIR/../dist"
REPO_DIR="$( cd "$SCRIPT_DIR/../../../../" && pwd )"
PROTO_DIR="$( cd "$REPO_DIR/mullvad-management-interface/proto" && pwd )"

source "$REPO_DIR/scripts/utils/host"

# If the host system is ARM-based we must specify the guest platform,
# otherwise an error will be throw about mismatching architecture
if [[ $HOST == "aarch64"* ]]; then
    RUN_COMMANDS=(--platform linux/amd64)
fi

set -x
exec "$CONTAINER_RUNNER" run --rm -it "${RUN_COMMANDS[@]}" \
    -v "$PROTO_DIR:/proto:Z" \
    -v "$OUT_DIR:/proto-bindings:Z" \
    "$IMAGE_NAME"
