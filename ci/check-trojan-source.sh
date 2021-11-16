#!/usr/bin/env bash

# This script scans text and source code for bidirectional Unicode characters.
# See CVE-2021-42574. https://cve.mitre.org/cgi-bin/cvename.cgi?name=CVE-2021-42574
# UTF-8 encoding is assumed.

# Pass the path to the directory to check as the first argument

set -eu

export LC_ALL=en_US.UTF-8

if [ $# -ne 1 ]; then
    echo "Usage: $0 <path>"
    exit 1
fi
cd "$1"

FILES=()
while IFS='' read -r line; do FILES+=("$line"); done < <( find . -type f -exec grep -Il . {} + )

CODEPOINT_REGEX=$( printf "\u202a\|\u202b\|\u202c\|\u202d\|\u202e\|\u2066\|\u2067\|\u2068\|\u2069" )

matched=0
for file in "${FILES[@]}"; do
    if grep -q "${CODEPOINT_REGEX}" "$file"; then
        echo "Found potentially malicious unicode code points in $file"
        matched=1
    fi
done

if [[ "$matched" == 0 ]]; then
    echo "No potentially malicious unicode found"
fi

exit $matched
