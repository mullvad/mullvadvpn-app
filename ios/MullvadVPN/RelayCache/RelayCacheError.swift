//
//  RelayCacheError.swift
//  RelayCacheError
//
//  Created by pronebird on 27/07/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension RelayCache {

    /// Error emitted by RelayCache cluster.
    enum Error: ChainedError {
        case readCache(Swift.Error)
        case readPrebundledRelays(Swift.Error)
        case decodePrebundledRelays(Swift.Error)
        case writeCache(Swift.Error)
        case encodeCache(Swift.Error)
        case decodeCache(Swift.Error)
        case rest(REST.Error)

        var errorDescription: String? {
            switch self {
            case .encodeCache:
                return "Encode cache error."
            case .decodeCache:
                return "Decode cache error."
            case .readCache:
                return "Read cache error."
            case .readPrebundledRelays:
                return "Read pre-bundled relays error."
            case .decodePrebundledRelays:
                return "Decode pre-bundled relays error."
            case .writeCache:
                return "Write cache error."
            case .rest:
                return "REST error."
            }
        }
    }

}
