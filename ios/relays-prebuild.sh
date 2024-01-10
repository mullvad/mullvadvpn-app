#!/usr/bin/env bash

if [ -z "$PROJECT_DIR" ]; then
  echo "This script is intended to be executed by Xcode"
  exit 1
fi

RELAYS_FILE="$PROJECT_DIR/MullvadREST/Assets/relays.json"

if [ "$CONFIGURATION" == "Staging" ]; then
  API_ENDPOINT="api.stagemole.eu"
else
  API_ENDPOINT="api.mullvad.net"
fi

if [ "$CONFIGURATION" == "Release" ]; then
  echo "Remove relays file"
  if [ -f "$RELAYS_FILE" ]; then
    rm "$RELAYS_FILE"
  else
    echo "Relays file does not exist"
  fi
fi

if [ ! -f "$RELAYS_FILE" ]; then
  echo "Download relays file"
  curl https://"$API_ENDPOINT"/app/v1/relays -s -o "$RELAYS_FILE"
fi
