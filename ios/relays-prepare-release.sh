#!/usr/bin/env bash

API_ENDPOINT="api.mullvad.net"
RELAYS_FILE="MullvadREST/Assets/relays.json"

echo "Download relays file"
curl https://"$API_ENDPOINT"/app/v1/relays -s -o "$RELAYS_FILE"

git add -f "$RELAYS_FILE"
git commit -m "Add updated relay list to release build"