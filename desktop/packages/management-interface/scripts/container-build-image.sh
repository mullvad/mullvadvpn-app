#!/usr/bin/env bash

# Build the container image which can generate JS/TS bindings from proto files.
#
# Please refer to the documentation for more information on how to use this
# script.

set -eu

CONTAINER_RUNNER=${CONTAINER_RUNNER:-"podman"}

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
CONTAINER_DIR="$( cd "$SCRIPT_DIR/.." && pwd )"
IMAGE_NAME="grpc-js-ts-bindings"

exec "$CONTAINER_RUNNER" build "$CONTAINER_DIR" -t $IMAGE_NAME "$@"
