#!/usr/bin/env bash

# This script runs within the node gRPC bindings container
# and is used to generate JS/TS bindings for each proto file in the
# IN_DIR folder and then output them to the OUT_DIR folder.
#
# This script is used as the entrypoint of the node gRPC bindings container.

set -eu

NODE_BINDINGS_DEPENDENCIES_DIR="/node-grpc-bindings/bindings-dependencies"
GRPC_TOOLS_NODE_PROTOC="$NODE_BINDINGS_DEPENDENCIES_DIR/grpc_tools_node_protoc"
TS_PROTOC_PLUGIN="$NODE_BINDINGS_DEPENDENCIES_DIR/node_modules/.bin/protoc-gen-ts"

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

