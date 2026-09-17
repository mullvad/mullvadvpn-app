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

/// A struct that represents the relay cache in memory
public struct CachedRelays: Codable, Equatable, Sendable {
    /// E-tag returned by server
    public let etag: String?

    /// The relay list stored within the cache entry
    public let relays: REST.ServerRelaysResponse

    /// The date when this cache was last updated
    public let updatedAt: Date

    public init(etag: String? = nil, relays: REST.ServerRelaysResponse, updatedAt: Date) {
        self.etag = etag
        self.relays = relays
        self.updatedAt = updatedAt
    }

    /// Returns true if the relay list contains no usable relays
    public var isEmpty: Bool {
        relays.isEmpty
    }

    /// Empty relay list used when prebundled file is empty (Debug/Staging builds)
    public static var empty: CachedRelays {
        CachedRelays(
            etag: nil,
            relays: .empty,
            updatedAt: Date(timeIntervalSince1970: 0)
        )
    }
}
