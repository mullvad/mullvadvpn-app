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
  curl https://api.mullvad.net/app/v1/relays -s -o "$RELAYS_FILE"
fi
