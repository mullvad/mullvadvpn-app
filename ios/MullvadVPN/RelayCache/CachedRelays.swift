//
//  CachedRelays.swift
//  CachedRelays
//
//  Created by pronebird on 27/07/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension RelayCache {
    /// A struct that represents the relay cache on disk
    struct CachedRelays: Codable {
        /// E-tag returned by server
        var etag: String?

        /// The relay list stored within the cache entry
        var relays: REST.ServerRelaysResponse

        /// The date when this cache was last updated
        var updatedAt: Date
    }
}
