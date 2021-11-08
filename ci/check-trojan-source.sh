#!/usr/bin/env bash

# This script scans text and source code for bidirectional Unicode characters.
# See CVE-2021-42574. https://cve.mitre.org/cgi-bin/cvename.cgi?name=CVE-2021-42574
# UTF-8 encoding is assumed.

set -u

export LC_ALL=en_US.UTF-8

SCRIPT_RELPATH="./$(basename "$(pwd)")/$(basename "${BASH_SOURCE[0]}")"
cd "$(dirname "${BASH_SOURCE[0]}")/.." || exit 1

# List all non-binary files
FILES=()
while IFS='' read -r line; do FILES+=("$line"); done < <( find . -type f -not -path "$SCRIPT_RELPATH" -exec grep -Il . {} + )

CODEPOINT_REGEX=$( printf "\u202a\|\u202b\|\u202c\|\u202d\|\u202e\|\u2066\|\u2067\|\u2068\|\u2069" )

function unicode_scan() {
    grep -q "${CODEPOINT_REGEX}"
}

################################################################################
# Sanity check.
################################################################################

UNSAFE_STR="nonsense ‪"
SAFE_STR="nonsense x"

if ! unicode_scan <<< "${UNSAFE_STR}"; then
    echo "Failed to detect code point in test string"
    exit 1
fi

if unicode_scan <<< "${SAFE_STR}"; then
    echo "Incorrectly detected code point in test string"
    exit 1
fi

################################################################################
# Scan all files for the malicious code points.
################################################################################

matched=0

echo "Scanning files: ${FILES[*]}"

for file in "${FILES[@]}"; do
    if unicode_scan "$file"; then
        echo "Found code points in $file"
        matched=1
    fi
done

exit $matched
