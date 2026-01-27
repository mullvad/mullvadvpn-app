#!/usr/bin/env xcrun --sdk macosx swift

import Foundation

guard let projectDir = ProcessInfo.processInfo.environment["PROJECT_DIR"] else {
    fputs("This script is intended to be executed by Xcode\n", stderr)
    exit(1)
}

let relaysFile = "\(projectDir)/MullvadREST/Assets/relays.json"
let configuration = ProcessInfo.processInfo.environment["CONFIGURATION"] ?? ""

// For Release builds: require pre-committed file with valid relay data
if configuration == "Release" || configuration == "MockRelease" {
    let fileURL = URL(fileURLWithPath: relaysFile)

    guard FileManager.default.fileExists(atPath: relaysFile) else {
        fputs("Error: No file found at \(relaysFile)\n", stderr)
        exit(1)
    }

    print("Validating list")

    guard let data = try? Data(contentsOf: fileURL), !data.isEmpty else {
        fputs("Error: Relay file at \(relaysFile) is empty or unreadable\n", stderr)
        exit(1)
    }

    guard let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any] else {
        fputs("Error: Invalid JSON in relay file \(relaysFile)\n", stderr)
        exit(1)
    }

    guard let wireguard = json["wireguard"] as? [String: Any],
          let relays = wireguard["relays"] as? [[String: Any]],
          !relays.isEmpty else {
        fputs("Error: Relay file contains no WireGuard relays\n", stderr)
        exit(1)
    }

    print("Relay file validated: \(relays.count) WireGuard relays found")
}
