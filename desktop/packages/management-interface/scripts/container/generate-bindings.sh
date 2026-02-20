#!/usr/bin/env bash

# This script runs within the management interface container
# and is used to generate JS bindings for each proto files in the
# /build/proto folder and then output them to the /build/dist folder
#
# This script is invoked by the scripts/container-run-generate-bindings.sh
# script. Please refer to the documentation for more information on how to use
# this script.

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
NODE_MODULES_BIN_DIR="$( cd "$SCRIPT_DIR/node_modules/.bin" && pwd )"
GRPC_TOOLS_NODE_PROTOC="$NODE_MODULES_BIN_DIR/grpc_tools_node_protoc"
TS_PROTOC_PLUGIN="$NODE_MODULES_BIN_DIR/protoc-gen-ts"

PROTO_DIR="/build/proto"
OUT_DIR="/build/dist"

cd "$SCRIPT_DIR"

for PROTO_FILE in "$PROTO_DIR"/*.proto; do
    PROTO_FILENAME="$(basename "$PROTO_FILE")"

    "$GRPC_TOOLS_NODE_PROTOC" \
        --js_out=import_style=commonjs,binary:$OUT_DIR \
        --grpc_out=grpc_js:$OUT_DIR \
        --proto_path=$PROTO_DIR \
        "$PROTO_DIR/$PROTO_FILENAME"

    "$GRPC_TOOLS_NODE_PROTOC" \
        --plugin=protoc-gen-ts="$TS_PROTOC_PLUGIN" \
        --ts_out=grpc_js:$OUT_DIR \
        --proto_path=$PROTO_DIR \
        "$PROTO_DIR/$PROTO_FILENAME"
done
