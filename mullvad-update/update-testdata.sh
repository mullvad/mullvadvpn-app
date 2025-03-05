#!/usr/bin/env bash

# This script updates ./test-version-response.json by signing ./unsigned-response.json.
# The JSON data is used by several unit tests.

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

# Update test-version-response from

secret="a459c1ee4f128780592b61454786cb289b38034a3ac1c7860e6e62187ac6e9a9"
pubkey="BB4EF63FFDCC6BD5A19C30CD23B9DE03099407A04463418F17AE338B98AA09D4"

echo "secret: $secret"
echo "pubkey: $pubkey"

cargo r --bin mullvad-version-metadata --features sign --features client \
    sign --file ./unsigned-response.json --secret $secret > test-version-response.json

echo -n "$pubkey" > test-pubkey
