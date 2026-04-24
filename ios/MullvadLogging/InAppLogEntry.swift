//
//  InAppLogEntry.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-04-24.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

public struct InAppLogEntry: Sendable, Codable, CustomStringConvertible {
    public enum Process: String, Codable, Sendable, CaseIterable {
        case app = "App"
        case packetTunnel = "PacketTunnel"
    }

    public let process: Process
    public let timestamp: String
    public let label: String
    public let message: String

    public var description: String {
        "[\(timestamp)][\(process)][\(label)]\n\(message)"
    }

    public init(process: Process, timestamp: String, label: String, message: String) {
        self.process = process
        self.timestamp = timestamp
        self.label = label
        self.message = message
    }
}
