#!/usr/bin/env bash

if [ -z "$PROJECT_DIR" ]; then
  echo "This script is intended to be executed by Xcode"
  exit 1
fi

RELAYS_FILE="$PROJECT_DIR/MullvadREST/Assets/relays.json"

# Do not download the file for release builds, a different script will take care of that.
if [ "$CONFIGURATION" == "Release" ]; then
  # Fail loudly if the required file is not already present.
  if [ ! -f "$RELAYS_FILE" ]; then
    echo "No file found at $RELAYS_FILE"
    exit 1
  fi
  return 0
fi

BACKUP_FILE="$CONFIGURATION_TEMP_DIR/relays.json"

if [ "$CONFIGURATION" == "Staging" ]; then
  API_ENDPOINT="api.stagemole.eu"
else
  API_ENDPOINT="api.mullvad.net"
fi

if [ ! -f "$BACKUP_FILE" ]; then
  echo "Downloading relays file for $CONFIGURATION"
  curl https://"$API_ENDPOINT"/app/v1/relays -s -o "$BACKUP_FILE"
fi

cp "$BACKUP_FILE" "$RELAYS_FILE"
