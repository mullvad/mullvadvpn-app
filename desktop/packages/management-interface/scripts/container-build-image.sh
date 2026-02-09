#!/usr/bin/env bash

# Build the container image which generates JS bindings for proto files.

set -eu

CONTAINER_RUNNER=${CONTAINER_RUNNER:-"docker"}

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
CONTAINER_DIR="$SCRIPT_DIR/.."
IMAGE_NAME=$(cat "$SCRIPT_DIR/../management-interface-container-image.txt")

cd "$SCRIPT_DIR"

exec "$CONTAINER_RUNNER" build $CONTAINER_DIR -t $IMAGE_NAME "$@"
