#!/usr/bin/env bash

# This script is used to build the management interface proto files to later use them on the platforms
# unsupported by grpc-tools (e.g. Apple Silicon).

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROTO_BUILD_DIR="$SCRIPT_DIR/mi_proto"

npm ci
npm run build-proto

mkdir -p "$PROTO_BUILD_DIR"
cp $SCRIPT_DIR/../../gui/src/main/management_interface/* $PROTO_BUILD_DIR
cp $SCRIPT_DIR/../../gui/build/src/main/management_interface/* $PROTO_BUILD_DIR

echo ""
echo "Management interface proto files built successfully."
echo "Build directory: $PROTO_BUILD_DIR"
