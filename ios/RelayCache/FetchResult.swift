//
//  FetchResult.swift
//  RelayCache
//
//  Created by Sajad Vishkai on 2022-10-21.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension RelayCache {
    /// Type describing the result of an attempt to fetch the new relay list from server.
    public enum FetchResult: CustomStringConvertible {
        /// Request to update relays was throttled.
        case throttled

        /// Refreshed relays but the same content was found on remote.
        case sameContent

        /// Refreshed relays with new content.
        case newContent

        public var description: String {
            switch self {
            case .throttled:
                return "throttled"
            case .sameContent:
                return "same content"
            case .newContent:
                return "new content"
            }
        }
    }

    public struct NoCachedRelaysError: LocalizedError {
        public var errorDescription: String? {
            return "Relay cache is empty."
        }

        public init() {}
    }
}
