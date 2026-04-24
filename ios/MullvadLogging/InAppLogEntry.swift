//
//  InAppLogEntry.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-04-24.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

public struct InAppLogEntry: Sendable, CustomStringConvertible {
    public let timestamp: String
    public let label: String
    public let message: String

    public var description: String {
        "\(timestamp) \(label)\n\(message)"
    }

    public init(timestamp: String, label: String, message: String) {
        self.timestamp = timestamp
        self.label = label
        self.message = message
    }
}
