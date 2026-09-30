#!/usr/bin/env bash

# Check if any string or plural is empty across all locales.

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR/.."

PATTERNS=(
    # Blank, or only whitespace/quotes, e.g. <string name="x"></string> or <string name="x">""</string>
    -e '<string[^>]*>[[:space:]"]*</string>'
    # Same for plurals, e.g. <item quantity="one"></item> or <item quantity="one">" "</item>
    -e '<item[^>]*>[[:space:]"]*</item>'
    # Self-closing string, e.g. <string name="x"/>
    -e '<string[^>]*/>'
    # Self-closing plural, e.g. <item quantity="one"/>
    -e '<item[^>]*/>'
)

if git grep -n "${PATTERNS[@]}" -- '*/res/values*/strings*.xml' '*/res/values*/plurals.xml'; then
    echo "Error: Empty Android string resources found."
    exit 1
fi
