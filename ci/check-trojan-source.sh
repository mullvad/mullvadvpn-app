#!/usr/bin/env bash

# Script for detecting suspicious code points in source code and text.
# It assumes UTF-8 encoding.

set -u

export LC_ALL=en_US.UTF-8

CODEPOINT_REGEX=$( printf "\u202a\|\u202b\|\u202c\|\u202d\|\u202e\|\u2066\|\u2067\|\u2068\|\u2069" )

SCRIPT_RELPATH="./$(realpath --relative-to="$(dirname "${BASH_SOURCE[0]}")/.." "${BASH_SOURCE[0]}")"
cd "$(dirname "${BASH_SOURCE[0]}")/.." || exit 1

# List all non-binary files
FILES=()
while IFS='' read -r line; do FILES+=("$line"); done < <( find . -type f -not -path "$SCRIPT_RELPATH" -exec grep -Il . {} + )

################################################################################
# Sanity check.
################################################################################

UNSAFE_STR="nonsense ‪"
SAFE_STR="nonsense x"

if ! echo "$UNSAFE_STR" | grep -q "${CODEPOINT_REGEX}"; then
    echo "Failed to detect code point in test string"
    exit 1
fi

if echo "$SAFE_STR" | grep -q "${CODEPOINT_REGEX}"; then
    echo "Incorrectly detected code point in test string"
    exit 1
fi

################################################################################
# Scan all files for the malicious code points.
################################################################################

matched=0

echo "Scanning files: ${FILES[*]}"

for file in "${FILES[@]}"; do
    if grep -q "${CODEPOINT_REGEX}" "$file"; then
        echo "Found code points in $file"
        matched=1
    fi
done

exit $matched
