#!/bin/sh

if [ -z "$PROJECT_DIR" ]; then
  echo "This script is intended to be executed by Xcode"
  exit 1
fi

ASSETS_DIR_PATH="$PROJECT_DIR/Assets"

RELAYS_FILE="$ASSETS_DIR_PATH/relays.json"
API_IP_ADDRESS_LIST_FILE="$ASSETS_DIR_PATH/api-ip-address.json"

if [ $CONFIGURATION == "Release" ]; then
  echo "Remove relays file"
  if [ -f "$RELAYS_FILE" ]; then
    rm "$RELAYS_FILE"
  else
    echo "Relays file does not exist"
  fi

  echo "Remove API address list file"
  if [ -f "$API_IP_ADDRESS_LIST_FILE" ]; then
    rm "$API_IP_ADDRESS_LIST_FILE"
  else
    echo "API IP address list file does not exist"
  fi
fi

if [ ! -f "$RELAYS_FILE" ]; then
  echo "Download relays file"
  curl https://api.mullvad.net/app/v1/relays -s -o "$RELAYS_FILE"
fi

if [ ! -f "$API_IP_ADDRESS_LIST_FILE" ]; then
  echo "Download API address list"
  curl https://api.mullvad.net/app/v1/api-addrs -s -o "$API_IP_ADDRESS_LIST_FILE"
fi