#!/usr/bin/env bash

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

NODE_MODULES_DIR="$(realpath ../node_modules/.bin)"
PROTO_DIR="../../mullvad-daemon/proto"
PROTO_FILENAME="management_interface.proto"
DESTINATION_DIR="../build/src/main/management_interface"
TYPES_DESTINATION_DIR="../src/main/management_interface"

TS_PROTOC_PLUGIN="$NODE_MODULES_DIR/protoc-gen-ts"
if [[ "$(uname -s)" == "MINGW"* ]]; then
  TS_PROTOC_PLUGIN="$TS_PROTOC_PLUGIN.cmd"
fi

mkdir -p $DESTINATION_DIR
mkdir -p $TYPES_DESTINATION_DIR

"$NODE_MODULES_DIR/grpc_tools_node_protoc" \
    --js_out=import_style=commonjs,binary:$DESTINATION_DIR \
    --grpc_out=generate_package_definition:$DESTINATION_DIR \
    --proto_path=$PROTO_DIR \
    $PROTO_DIR/$PROTO_FILENAME

"$NODE_MODULES_DIR/grpc_tools_node_protoc" \
    --plugin=protoc-gen-ts=$TS_PROTOC_PLUGIN \
    --ts_out=generate_package_definition:$TYPES_DESTINATION_DIR \
    --proto_path=$PROTO_DIR \
    $PROTO_DIR/$PROTO_FILENAME

