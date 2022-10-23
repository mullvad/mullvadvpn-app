#!/usr/bin/env bash

if [ -z "$PROJECT_DIR" ]; then
  echo "This script is intended to be executed by Xcode"
  exit 1
fi

RELAYS_FILE="$PROJECT_DIR/RelayCache/Assets/relays.json"

if [ $CONFIGURATION == "Release" ]; then
  echo "Remove relays file"
  if [ -f "$RELAYS_FILE" ]; then
    rm "$RELAYS_FILE"
  else
    echo "Relays file does not exist"
  fi
fi

if [ ! -f "$RELAYS_FILE" ]; then
  echo "Download relays file"
  curl https://api.mullvad.net/app/v1/relays -s -o "$RELAYS_FILE"
fi
