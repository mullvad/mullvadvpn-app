#!/bin/sh

if [ -z "$PROJECT_DIR" ]; then
  echo "This script is intended to be executed by Xcode"
  exit 1
fi

RELAYS_FILE="$PROJECT_DIR/Assets/relays.json"

if [ $CONFIGURATION == "Release" ]; then
  echo "Remove relays file"
  rm "$RELAYS_FILE" || true
fi

if [ ! -f "$RELAYS_FILE" ]; then
  echo "Download relays file"
  curl https://api.mullvad.net/rpc/ \
    -d '{"jsonrpc": "2.0", "id": "0", "method": "relay_list_v3"}' \
    --header "Content-Type: application/json" | jq -c .result > "$RELAYS_FILE"
fi
