#!/usr/bin/env bash

# Update the image name for the container which generates JS/TS bindings from
# proto files. Please refer to the documentation for more information on how
# to use this script.

set -eu

CONTAINER_RUNNER=${CONTAINER_RUNNER:-"docker"}

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
CONTAINER_DIR="$( cd "$SCRIPT_DIR/.." && pwd )"

COMMIT_SHA="$( git rev-parse --short HEAD )"
IMAGE_NAME="management-interface-container-image-$COMMIT_SHA"
IMAGE_NAME_FILENAME="$CONTAINER_DIR/management-interface-container-image.txt"

cd "$SCRIPT_DIR"

echo $IMAGE_NAME > $IMAGE_NAME_FILENAME
