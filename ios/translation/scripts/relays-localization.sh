#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$SCRIPT_DIR/../.."
INPUT_FILE="${1:-$SCRIPT_DIR/relays.json}"
OUTPUT_FILE="${2:-$PROJECT_DIR/Assets/RelayLocationList.swift}"

if [ ! -f "$INPUT_FILE" ]; then
  echo "Error: Input file '$INPUT_FILE' not found."
  exit 1
fi

mkdir -p "$(dirname "$OUTPUT_FILE")"

extract_key() {
  local key=$1
  grep -oE "\"$key\"\\s*:\\s*\"[^\"]+\"" "$INPUT_FILE" |
    sed -E "s/.*\"$key\"\\s*:\\s*\"([^\"]+)\".*/\\1/"
}

countries=$(extract_key "country")
cities=$(extract_key "city")

echo "Updating '$(basename "$OUTPUT_FILE")'."

all_locations=$(printf "%s\n%s\n" "$countries" "$cities" | awk '!seen[tolower($0)]++')

{
  echo "// Auto-generated from $(basename "$INPUT_FILE")"
  echo
  echo "import Foundation"
  echo
  echo "let relayLocationList: [String: String] = ["
  while read -r name; do
    [ -z "$name" ] && continue
    echo "    \"$name\": NSLocalizedString(\"$name\", comment: \"\"),"
  done <<<"$all_locations"
  echo "]"
} >"$OUTPUT_FILE"
