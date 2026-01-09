#!/usr/bin/env node

const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');
const os = require('os');

const SCRIPT_DIR = __dirname;
const NODE_MODULES_BIN = path.join(SCRIPT_DIR, '../../node_modules/.bin');
const OUT_DIR = path.join(SCRIPT_DIR, 'dist');
const PROTO_DIR = path.join(SCRIPT_DIR, '../../../mullvad-management-interface/proto');
const PROTO_FILENAME = 'management_interface.proto';

// Determine protoc plugin name based on platform
const isWindows = os.platform() === 'win32';
const TS_PROTOC_PLUGIN = path.join(NODE_MODULES_BIN, isWindows ? 'protoc-gen-ts.cmd' : 'protoc-gen-ts');
const GRPC_TOOLS_NODE_PROTOC = path.join(NODE_MODULES_BIN, isWindows ? 'grpc_tools_node_protoc.cmd' : 'grpc_tools_node_protoc');

// Create output directory
if (!fs.existsSync(OUT_DIR)) {
  fs.mkdirSync(OUT_DIR, { recursive: true });
}

// Check for ARM64 Linux special case
const arch = os.arch();
const platform = os.platform();
const MANAGEMENT_INTERFACE_PROTO_BUILD_DIR = process.env.MANAGEMENT_INTERFACE_PROTO_BUILD_DIR;

if (platform === 'linux' && (arch === 'arm64' || arch === 'aarch64')) {
  if (MANAGEMENT_INTERFACE_PROTO_BUILD_DIR) {
    // Copy pre-built files
    const files = fs.readdirSync(MANAGEMENT_INTERFACE_PROTO_BUILD_DIR);
    files.filter(f => f.endsWith('.js') || f.endsWith('.ts')).forEach(file => {
      fs.copyFileSync(
        path.join(MANAGEMENT_INTERFACE_PROTO_BUILD_DIR, file),
        path.join(SCRIPT_DIR, file)
      );
    });
  } else {
    console.error('Building management interface proto files on aarch64 is not supported');
    console.error('(see https://github.com/grpc/grpc-node/issues/1497).');
    console.error('Please build the proto files on another platform using build-proto.sh script,');
    console.error('and set MANAGEMENT_INTERFACE_PROTO_BUILD_DIR environment variable to the directory of the build.');
    process.exit(1);
  }
} else {
  // Run protoc for JavaScript output
  try {
    execSync(`"${GRPC_TOOLS_NODE_PROTOC}" --js_out=import_style=commonjs,binary:${OUT_DIR} --grpc_out=grpc_js:${OUT_DIR} --proto_path=${PROTO_DIR} ${path.join(PROTO_DIR, PROTO_FILENAME)}`, {
      stdio: 'inherit',
      shell: true
    });
  } catch (error) {
    console.error('Failed to generate JavaScript protobuf files');
    process.exit(1);
  }

  // Run protoc for TypeScript output
  try {
    execSync(`"${GRPC_TOOLS_NODE_PROTOC}" --plugin=protoc-gen-ts="${TS_PROTOC_PLUGIN}" --ts_out=grpc_js:${OUT_DIR} --proto_path=${PROTO_DIR} ${path.join(PROTO_DIR, PROTO_FILENAME)}`, {
      stdio: 'inherit',
      shell: true
    });
  } catch (error) {
    console.error('Failed to generate TypeScript protobuf files');
    process.exit(1);
  }
}

console.log('Proto build completed successfully');
