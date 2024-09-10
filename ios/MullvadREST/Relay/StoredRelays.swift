//
//  StoredRelays.swift
//  MullvadVPNUITests
//
//  Created by Jon Petersson on 2024-09-09.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// A struct that represents the relay cache on disk
public struct StoredRelays: Codable, Equatable {
    /// E-tag returned by server
    public let etag: String?

    /// The raw relay JSON data stored within the cache entry
    public let rawData: Data

    /// The date when this cache was last updated
    public let updatedAt: Date

    /// Relays parsed from the JSON data
    public var relays: REST.ServerRelaysResponse {
        get throws {
            try REST.Coding.makeJSONDecoder().decode(REST.ServerRelaysResponse.self, from: rawData)
        }
    }

    /// `CachedRelays` representation
    public var toCachedRelays: CachedRelays {
        get throws {
            try CachedRelays(etag: etag, relays: relays, updatedAt: updatedAt)
        }
    }

    public init(etag: String? = nil, rawData: Data, updatedAt: Date) {
        self.etag = etag
        self.rawData = rawData
        self.updatedAt = updatedAt
    }

    public init(cachedRelays: CachedRelays) throws {
        etag = cachedRelays.etag
        rawData = try REST.Coding.makeJSONEncoder().encode(cachedRelays.relays)
        updatedAt = cachedRelays.updatedAt
    }
}
