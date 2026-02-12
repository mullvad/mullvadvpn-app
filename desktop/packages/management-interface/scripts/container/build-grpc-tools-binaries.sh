#!/usr/bin/env bash

# This script runs within the management interface container
# and is used to build the platform specific gRPC binaries which
# can then be used to generate bindings from proto files.
#
# This build script has 3 main purposes.
# 1. Compiles the protoc/grpc_node binaries and copies them to the node_modules/.bin folder.
# 2. Copies google protobuf files to the node_modules/.bin folder
# 3. Copies node protoc files to the node_modules/.bin folder
#
# Please refer to the documentation for more information on how to use
# this script.

set -eu

GRPC_TOOLS_BASE="/build/grpc-node/packages/grpc-tools"
PROTOBUF_BASE=$GRPC_TOOLS_BASE/deps/protobuf
OUT_DIR="/build/node_modules/.bin"

cd $GRPC_TOOLS_BASE

# 1. Compile the protoc/grpc_node binaries.
# This cmake command is based on the grpc-tools package within grpc-node
# (grpc-node/packages/grpc-tools/build_binaries.sh), but is stripped down to
# only build for the current platform/arch.
cmake . && cmake --build . --target clean && cmake --build . -- -j 12

# Note: protoc is a symlink to a versioned file, hence why we need the -L flag.
cp -L "$PROTOBUF_BASE/protoc" "$OUT_DIR/protoc"
cp "$GRPC_TOOLS_BASE/grpc_node_plugin" "$OUT_DIR/grpc_node_plugin"

# 2. Gather all google protobuf files in the bin folder in grpc-tools
# and then copy them to the node_modules/.bin folder
node "$GRPC_TOOLS_BASE/copy_well_known_protos.js"
cp -r "$GRPC_TOOLS_BASE/bin/google" "$OUT_DIR/google"

# 3. Copy the "bin" files defined in grpc-tools package.json
# (grpc-node/packages/grpc-tools/package.json) to the node_modules/.bin folder.
#
# TODO: We could use jq to read these "bin" files from the package.json
cp "$GRPC_TOOLS_BASE/bin/protoc.js" "$OUT_DIR/grpc_tools_node_protoc"
cp "$GRPC_TOOLS_BASE/bin/protoc_plugin.js" "$OUT_DIR/grpc_tools_node_protoc_plugin"
