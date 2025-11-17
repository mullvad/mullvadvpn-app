#!/usr/bin/env bash
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

RELAYS_FILE=${RELAYS_FILE:-"$SCRIPT_DIR/relays.json"}

API_ENDPOINT="api.mullvad.net"

echo "Downloading relays file"
curl https://"$API_ENDPOINT"/app/v1/relays -s -o "$RELAYS_FILE"
