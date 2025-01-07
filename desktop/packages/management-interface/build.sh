#!/usr/bin/env bash

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

ARCH="$(uname -m)"
PLATFORM="$(uname -s)"
MANAGEMENT_INTERFACE_PROTO_BUILD_DIR=${MANAGEMENT_INTERFACE_PROTO_BUILD_DIR:-}
NODE_MODULES_DIR="$(cd ../../node_modules/.bin && pwd)"
OUT_DIR="dist"
PROTO_DIR="../../../mullvad-management-interface/proto"
PROTO_FILENAME="management_interface.proto"

TS_PROTOC_PLUGIN="$NODE_MODULES_DIR/protoc-gen-ts"
if [[ "$(uname -s)" == "MINGW"* ]]; then
  TS_PROTOC_PLUGIN="$TS_PROTOC_PLUGIN.cmd"
fi

mkdir -p $OUT_DIR

if [[ "$PLATFORM" == "Linux" && ("${ARCH,,}" == "arm64" || "${ARCH,,}" == "aarch64") ]]; then
    if [[ -n "${MANAGEMENT_INTERFACE_PROTO_BUILD_DIR}" ]]; then
      cp "$MANAGEMENT_INTERFACE_PROTO_BUILD_DIR"/*.js .
      cp "$MANAGEMENT_INTERFACE_PROTO_BUILD_DIR"/*.ts .
    else
      >&2 echo "Building management interface proto files on aarch64 is not supported"
      >&2 echo "(see https://github.com/grpc/grpc-node/issues/1497)."
      >&2 echo "Please build the proto files on another platform using build-proto.sh script,"
      >&2 echo "and set MANAGEMENT_INTERFACE_PROTO_BUILD_DIR environment variable to the directory of the build."
      exit 1
    fi
else
  "$NODE_MODULES_DIR/grpc_tools_node_protoc" \
      --js_out=import_style=commonjs,binary:$OUT_DIR \
      --grpc_out=grpc_js:$OUT_DIR \
      --proto_path=$PROTO_DIR \
      $PROTO_DIR/$PROTO_FILENAME

  "$NODE_MODULES_DIR/grpc_tools_node_protoc" \
      --plugin=protoc-gen-ts="$TS_PROTOC_PLUGIN" \
      --ts_out=grpc_js:$OUT_DIR \
      --proto_path=$PROTO_DIR \
      $PROTO_DIR/$PROTO_FILENAME
fi
