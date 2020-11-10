#!/usr/bin/env bash

echo "Updating API address cache..."
set -e

cargo +stable run --bin address_cache --release > dist-assets/api-ip-address.txt
