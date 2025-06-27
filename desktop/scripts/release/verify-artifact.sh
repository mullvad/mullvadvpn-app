#!/usr/bin/env bash

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

SIGNER_KEY_FILE="./mullvad-code-signing-key.asc"

filename="$1"

# We prefer sqv for PGP key verification as it a strict and easy-to-use implementation of PGP.
# gpg is also not suitable for use in scripting.
if ! sqv --keyring "$SIGNER_KEY_FILE" "$filename.asc" "$filename"; then
    echo ""
    echo "!!! INTEGRITY CHECKING FAILED !!!"
    rm "$filename" "$filename.asc"
    exit 1
fi
