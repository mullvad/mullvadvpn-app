#!/usr/bin/env bash

# This script runs within the node gRPC bindings container
# and is used to generate JS/TS bindings for each proto file in the
# IN_DIR folder and then output them to the OUT_DIR folder.
#
# This script is used as the entrypoint of the node gRPC bindings container.

set -eu

GRPC_TOOLS_NODE_PROTOC="/node-grpc-bindings/bindings-dependencies/grpc_tools_node_protoc"
TS_PROTOC_PLUGIN="/node-grpc-bindings/bindings-dependencies/node_modules/.bin/protoc-gen-ts"

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
