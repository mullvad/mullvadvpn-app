#!/usr/bin/env bash

echo "Updating relay list..."
set -e

cargo run --bin relay_list --release > dist-assets/relays.json
