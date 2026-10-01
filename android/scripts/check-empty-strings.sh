#!/usr/bin/env bash

# Check if any string, plural or relay location is empty across all locales.

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR/.."

FILES=(
    '*/res/values*/strings*.xml'
    '*/res/values*/plurals.xml'
    '*/res/xml*/relay_locations.xml'
)

PATTERNS=(
    # Blank, or only whitespace/quotes
    # E.g. <string name="x"></string> or <string name="x">""</string>
    -e '<string[^>]*>[[:space:]"]*</string>'
    # Same for plurals.
    -e '<item[^>]*>[[:space:]"]*</item>'
    # Self-closing string, e.g. <string name="x"/>
    -e '<string[^>]*/>'
    # Same for plurals.
    -e '<item[^>]*/>'
)

if git grep -n "${PATTERNS[@]}" -- "${FILES[@]}"; then
    echo "Error: Empty Android string resources found."
    exit 1
fi
