#!/usr/bin/env bash

echo "Updating relay list..."
set -e
cargo run -p mullvad-rpc > dist-assets/relays.json
