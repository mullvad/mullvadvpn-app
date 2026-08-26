#!/usr/bin/env bash

API_ENDPOINT="api.mullvad.net"
RELAYS_FILE="MullvadREST/Assets/relays.json"

DIGEST=$(curl https://"$API_ENDPOINT"/trl/v1/timestamps/latest -s | head -n 1 | grep -o '"digest":"[^"]*' | grep -o '[^"]*$')

echo "Download relays file"
curl https://"$API_ENDPOINT"/trl/v1/data/"$DIGEST" -s -o "$RELAYS_FILE"

git add -f "$RELAYS_FILE"
git commit -m "Add updated relay list to release build"
