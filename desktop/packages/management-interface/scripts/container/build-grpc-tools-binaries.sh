#!/usr/bin/env bash

# This is a build script is based on the grpc-tools package within grpc-node
# (grpc-node/packages/grpc-tools/build_binaries.sh), but is stripped down to
# only build for the current platform/arch.
#
# It also handles copying the "bin" files defined in grpc-tools package.json
# (grpc-node/packages/grpc-tools/package.json) to the bin directory.
# which is required since we're no longer using grpc-tools as a npm
# dependency, and such we have to handle these things ourselves.

set -eu

GRPC_TOOLS_BASE="/build/grpc-node/packages/grpc-tools"
PROTOBUF_BASE=$GRPC_TOOLS_BASE/deps/protobuf
OUT_DIR="/build/node_modules/.bin"

cd $GRPC_TOOLS_BASE

cmake . && cmake --build . --target clean && cmake --build . -- -j 12

# Note: protoc is a symlink to a versioned file, hence why we need the -L flag.
cp -L "$PROTOBUF_BASE/protoc" "$OUT_DIR/protoc"
cp "$GRPC_TOOLS_BASE/grpc_node_plugin" "$OUT_DIR/grpc_node_plugin"
# Copy google protobuf protobuf to bin folder in grpc-tools 
node "$GRPC_TOOLS_BASE/copy_well_known_protos.js"
cp -r "$GRPC_TOOLS_BASE/bin/google" "$OUT_DIR/google"
# TODO: We could use jq to read these "bin" files from the package.json
cp "$GRPC_TOOLS_BASE/bin/protoc.js" "$OUT_DIR/grpc_tools_node_protoc"
cp "$GRPC_TOOLS_BASE/bin/protoc_plugin.js" "$OUT_DIR/grpc_tools_node_protoc_plugin"
