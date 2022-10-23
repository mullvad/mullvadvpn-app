//
//  CachedRelays.swift
//  CachedRelays
//
//  Created by pronebird on 27/07/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST

/// A struct that represents the relay cache on disk
public struct CachedRelays: Codable {
    /// E-tag returned by server
    public var etag: String?

    /// The relay list stored within the cache entry
    public var relays: REST.ServerRelaysResponse

    /// The date when this cache was last updated
    public var updatedAt: Date

    public init(etag: String? = nil, relays: REST.ServerRelaysResponse, updatedAt: Date) {
        self.etag = etag
        self.relays = relays
        self.updatedAt = updatedAt
    }
}
