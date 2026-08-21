// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation

public enum TunnelQuantumResistance: Codable, Sendable {
    case on
    case off

    private enum CodingKeys: String, CodingKey {
        case automatic, on, off
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        if container.contains(.automatic) {
            self = .on
            return
        }

        if container.contains(.on) {
            self = .on
            return
        }

        if container.contains(.off) {
            self = .off
            return
        }

        self = .on
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)

        switch self {
        case .on:
            try container.encode([String: String](), forKey: .on)

        case .off:
            try container.encode([String: String](), forKey: .off)
        }
    }
}

public extension TunnelQuantumResistance {
    /// A single source of truth for whether the current state counts as on
    var isEnabled: Bool {
        self == .on
    }
}
