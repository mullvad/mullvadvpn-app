#!/usr/bin/env bash

# This script runs within the gRPC node bindings container
# and is used to generate JS/TS bindings for each proto files in the
# /proto folder and then output them to the /proto-bindings folder
#
# This script is used as the entrypoint of the gRPC node bindings container.

set -eu

GRPC_TOOLS_NODE_PROTOC="/grpc-node-bindings-dependencies/grpc_tools_node_protoc"
TS_PROTOC_PLUGIN="/grpc-node-bindings-dependencies/protoc-gen-ts"

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
