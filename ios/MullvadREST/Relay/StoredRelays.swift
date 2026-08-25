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
    /// The sigsum digest for the cache entry
    public let digest: String?
    /// The timestamp of the sigsum digest
    public let timestamp: Int64?

    /// The raw relay JSON data stored within the cache entry
    public let rawData: Data

    /// The date when this cache was last updated
    public let updatedAt: Date

    /// `CachedRelays` representation, deserialized eagerly at construction time.
    public let cachedRelays: CachedRelays

    public init(digest: String? = nil, timestamp: Int64? = nil, rawData: Data, updatedAt: Date) throws {
        self.digest = digest
        self.timestamp = timestamp
        self.rawData = rawData
        self.updatedAt = updatedAt

        let relays = try REST.Coding.makeJSONDecoder().decode(
            REST.ServerRelaysResponse.self,
            from: rawData
        )
        cachedRelays = CachedRelays(digest: digest, timestamp: timestamp, relays: relays, updatedAt: updatedAt)
    }

    public init(cachedRelays: CachedRelays) throws {
        digest = cachedRelays.digest
        timestamp = cachedRelays.timestamp
        rawData = try REST.Coding.makeJSONEncoder().encode(cachedRelays.relays)
        updatedAt = cachedRelays.updatedAt

        self.cachedRelays = cachedRelays
    }

    // MARK: - Codable

    private enum CodingKeys: String, CodingKey {
        case digest, timestamp, rawData, updatedAt
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        digest = try? container.decode(String?.self, forKey: .digest)
        timestamp = try? container.decode(Int64?.self, forKey: .timestamp)
        rawData = try container.decode(Data.self, forKey: .rawData)
        updatedAt = try container.decode(Date.self, forKey: .updatedAt)
<<<<<<< HEAD
        // Eagerly deserialize relay data so the result is cached before
        // this value reaches the actor's cooperative thread. The
        // DeserializationCache.get() NSLock + JSON parsing would otherwise
        // block a cooperative thread during relay selection.
        let result = Result {
            try REST.Coding.makeJSONDecoder().decode(
                REST.ServerRelaysResponse.self,
                from: rawData
            )
        }.map { CachedRelays(digest: digest, timestamp: timestamp, relays: $0, updatedAt: updatedAt) }
        cache.set(result)
||||||| parent of 7d8a388b67 (Remove NSLock from FileCache and related files)
        // Eagerly deserialize relay data so the result is cached before
        // this value reaches the actor's cooperative thread. The
        // DeserializationCache.get() NSLock + JSON parsing would otherwise
        // block a cooperative thread during relay selection.
        let result = Result {
            try REST.Coding.makeJSONDecoder().decode(
                REST.ServerRelaysResponse.self,
                from: rawData
            )
        }.map { CachedRelays(etag: etag, relays: $0, updatedAt: updatedAt) }
        cache.set(result)
=======

        let relays = try REST.Coding.makeJSONDecoder().decode(
            REST.ServerRelaysResponse.self,
            from: rawData
        )
        cachedRelays = CachedRelays(etag: etag, relays: relays, updatedAt: updatedAt)
>>>>>>> 7d8a388b67 (Remove NSLock from FileCache and related files)
    }

    // MARK: - Equatable

    public static func == (lhs: StoredRelays, rhs: StoredRelays) -> Bool {
        lhs.rawData == rhs.rawData && lhs.updatedAt == rhs.updatedAt
    }
}
