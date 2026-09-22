// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

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
