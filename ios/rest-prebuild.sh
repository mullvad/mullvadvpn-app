#!/usr/bin/env bash

if [ -z "$PROJECT_DIR" ]; then
  echo "This script is intended to be executed by Xcode"
  exit 1
fi

API_IP_ADDRESS_LIST_FILE="$PROJECT_DIR/MullvadREST/Assets/api-ip-address.json"

if [ $CONFIGURATION == "Release" ]; then
  echo "Remove API address list file"
  if [ -f "$API_IP_ADDRESS_LIST_FILE" ]; then
    rm "$API_IP_ADDRESS_LIST_FILE"
  else
    echo "API IP address list file does not exist"
  fi
fi

if [ ! -f "$API_IP_ADDRESS_LIST_FILE" ]; then
  echo "Download API address list"
  curl https://api.mullvad.net/app/v1/api-addrs -s -o "$API_IP_ADDRESS_LIST_FILE"
fi
