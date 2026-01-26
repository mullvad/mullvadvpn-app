#!/usr/bin/env bash

if [ -z "$PROJECT_DIR" ]; then
  echo "This script is intended to be executed by Xcode"
  exit 1
fi

RELAYS_FILE="$PROJECT_DIR/MullvadREST/Assets/relays.json"

# For Release builds: require pre-committed file with valid relay data
if [ "$CONFIGURATION" == "Release" ] || [ "$CONFIGURATION" == "MockRelease" ]; then
  if [ ! -f "$RELAYS_FILE" ]; then
    echo "Error: No file found at $RELAYS_FILE"
    exit 1
  fi

    echo "Validating list"
  # Validate JSON structure and ensure it contains relays using Swift
  xcrun -sdk macosx swift - <<SWIFT
import Foundation

let filePath = "${RELAYS_FILE}"
let fileURL = URL(fileURLWithPath: filePath)

guard let data = try? Data(contentsOf: fileURL), !data.isEmpty else {
    fputs("Error: Relay file at \(filePath) is empty or unreadable\n", stderr)
    exit(1)
}

guard let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any] else {
    fputs("Error: Invalid JSON in relay file \(filePath)\n", stderr)
    exit(1)
}

guard let wireguard = json["wireguard"] as? [String: Any],
      let relays = wireguard["relays"] as? [[String: Any]],
      !relays.isEmpty else {
    fputs("Error: Relay file contains no WireGuard relays\n", stderr)
    exit(1)
}

print("Relay file validated: \(relays.count) WireGuard relays found")
SWIFT
   exit $?
fi
