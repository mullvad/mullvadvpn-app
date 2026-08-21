// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only
import MullvadTypes

public struct RecentConnections: Codable, Sendable, Equatable {
    private enum CodingKeys: String, CodingKey {
        case isEnabled
        case entryLocations
        case exitLocations
    }

    public let isEnabled: Bool
    public let entryLocations: [RelayConstraint<UserSelectedRelays>]
    public let exitLocations: [RelayConstraint<UserSelectedRelays>]

    public init(
        isEnabled: Bool = true,
        entryLocations: [RelayConstraint<UserSelectedRelays>] = [],
        exitLocations: [RelayConstraint<UserSelectedRelays>] = []
    ) {
        self.isEnabled = isEnabled
        self.entryLocations = entryLocations
        self.exitLocations = exitLocations
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        isEnabled = try container.decodeIfPresent(Bool.self, forKey: .isEnabled) ?? true
        entryLocations = Self.decodeLocations(from: container, forKey: .entryLocations)
        exitLocations = Self.decodeLocations(from: container, forKey: .exitLocations)
    }

    private static func decodeLocations(
        from container: KeyedDecodingContainer<CodingKeys>,
        forKey key: CodingKeys
    ) -> [RelayConstraint<UserSelectedRelays>] {
        if let locations = try? container.decode([RelayConstraint<UserSelectedRelays>].self, forKey: key) {
            return locations
        } else if let locations = try? container.decode([UserSelectedRelays].self, forKey: key) {
            return locations.map { .only($0) }
        }

        return []
    }
}
