#!/usr/bin/env bash

set -euo pipefail

RELAYS_FILE="MullvadREST/Assets/relays.json"

echo "Download relays file"
cargo run -q -p mullvad-api --bin relay-list > $RELAYS_FILE

git add -f "$RELAYS_FILE"
git commit -m "Add updated relay list to release build"
