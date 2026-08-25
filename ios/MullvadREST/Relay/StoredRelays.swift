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

/// A struct that represents the relay cache on disk
public struct StoredRelays: Codable, Equatable, Sendable {
    /// E-tag returned by server
    public let etag: String?

    /// The raw relay JSON data stored within the cache entry
    public let rawData: Data

    /// The date when this cache was last updated
    public let updatedAt: Date

    /// `CachedRelays` representation, deserialized eagerly at construction time.
    public let cachedRelays: CachedRelays

    public init(etag: String? = nil, rawData: Data, updatedAt: Date) throws {
        self.etag = etag
        self.rawData = rawData
        self.updatedAt = updatedAt

        let relays = try REST.Coding.makeJSONDecoder().decode(
            REST.ServerRelaysResponse.self,
            from: rawData
        )
        cachedRelays = CachedRelays(etag: etag, relays: relays, updatedAt: updatedAt)
    }

    public init(cachedRelays: CachedRelays) throws {
        etag = cachedRelays.etag
        rawData = try REST.Coding.makeJSONEncoder().encode(cachedRelays.relays)
        updatedAt = cachedRelays.updatedAt

        self.cachedRelays = cachedRelays
    }

    // MARK: - Codable

    private enum CodingKeys: String, CodingKey {
        case etag, rawData, updatedAt
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        etag = try container.decodeIfPresent(String.self, forKey: .etag)
        rawData = try container.decode(Data.self, forKey: .rawData)
        updatedAt = try container.decode(Date.self, forKey: .updatedAt)

        let relays = try REST.Coding.makeJSONDecoder().decode(
            REST.ServerRelaysResponse.self,
            from: rawData
        )
        cachedRelays = CachedRelays(etag: etag, relays: relays, updatedAt: updatedAt)
    }

    // MARK: - Equatable

    public static func == (lhs: StoredRelays, rhs: StoredRelays) -> Bool {
        lhs.etag == rhs.etag && lhs.rawData == rhs.rawData && lhs.updatedAt == rhs.updatedAt
    }
}
