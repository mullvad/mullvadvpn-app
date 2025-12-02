#!/bin/bash

# Configure npm
npm config set ignore-scripts true

# Install node dependencies
npm ci

# Create node_modules folder in mullvad-vpn
pushd packages/mullvad-vpn
test -d node_modules || mkdir node_modules
popd
