#!/usr/bin/env bash

# This script runs within the management interface container
# and is used to generate JS bindings for each proto files in the
# /proto folder and then output them to the /proto-bindings folder
#
# This script is invoked by the scripts/container-run-generate-bindings.sh
# script. Please refer to the documentation for more information on how to use
# this script.

set -eu

GRPC_TOOLS_NODE_PROTOC="/grpc-tools-binaries/grpc_tools_node_protoc"
TS_PROTOC_PLUGIN="/grpc-tools-binaries/protoc-gen-ts"

IN_DIR="/proto"
OUT_DIR="/proto-bindings"

for PROTO_FILE in "$IN_DIR"/*.proto; do
    PROTO_FILENAME="$(basename "$PROTO_FILE")"

    "$GRPC_TOOLS_NODE_PROTOC" \
        --js_out=import_style=commonjs,binary:$OUT_DIR \
        --grpc_out=grpc_js:$OUT_DIR \
        --proto_path=$IN_DIR \
        "$IN_DIR/$PROTO_FILENAME"

    "$GRPC_TOOLS_NODE_PROTOC" \
        --plugin=protoc-gen-ts="$TS_PROTOC_PLUGIN" \
        --ts_out=grpc_js:$OUT_DIR \
        --proto_path=$IN_DIR \
        "$IN_DIR/$PROTO_FILENAME"
done
