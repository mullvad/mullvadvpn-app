#!/usr/bin/env bash

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

PLATFORM="$(uname -s)-$(uname -m)"
MI_PROTO_BUILD_DIR=${MI_PROTO_BUILD_DIR:-}
NODE_MODULES_DIR="$(cd ../node_modules/.bin && pwd)"
PROTO_DIR="../../mullvad-management-interface/proto"
PROTO_FILENAME="management_interface.proto"
DESTINATION_DIR="../build/src/main/management_interface"
TYPES_DESTINATION_DIR="../src/main/management_interface"

TS_PROTOC_PLUGIN="$NODE_MODULES_DIR/protoc-gen-ts"
if [[ "$(uname -s)" == "MINGW"* ]]; then
  TS_PROTOC_PLUGIN="$TS_PROTOC_PLUGIN.cmd"
fi

mkdir -p $DESTINATION_DIR
mkdir -p $TYPES_DESTINATION_DIR

if [[ "${PLATFORM}" == "Darwin-arm64" ]]; then
    if [[ -n "${MI_PROTO_BUILD_DIR}" ]]; then
      cp $MI_PROTO_BUILD_DIR/*.js $DESTINATION_DIR
      cp $MI_PROTO_BUILD_DIR/*.ts $TYPES_DESTINATION_DIR
    else
      >&2 echo "Building management interface proto files on Apple Silicon is not supported"
      >&2 echo "(see https://github.com/grpc/grpc-node/issues/1497)."
      >&2 echo "Please build the proto files on another platform using build_mi_proto.sh script,"
      >&2 echo "and set MI_PROTO_BUILD_DIR environment variable to the directory of the build."
      exit 1
    fi
else
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
fi
