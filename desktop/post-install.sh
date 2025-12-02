#!/bin/bash

# Install electron
pushd node_modules/electron
npm run postinstall
popd

# Install grpc-tools
pushd node_modules/grpc-tools
npm run install
popd

# Build our own package: management interface
pushd packages/management-interface
npm run postinstall
popd

# Build our own package: nseventforwarder
pushd packages/nseventforwarder
npm run postinstall
popd

# Build our own package: windows-utils
pushd packages/windows-utils
npm run postinstall
popd
