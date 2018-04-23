#!/usr/bin/env bash

set -eu

echo "Cleaning filesystem of Rust artifacts"
cargo +stable clean

echo "Building WFPCTL"
WFP_BUILD_TARGETS='x64' WFP_BUILD_MODES='Release' bash build_wfp.sh

echo "Building Rust files"
cargo +stable build --release

echo "Updating relay list"
./target/release/list-relays > dist-assets/relays.json

echo "Installing JavaScript dependencies"
yarn install

echo "Building installer"
yarn pack:win
