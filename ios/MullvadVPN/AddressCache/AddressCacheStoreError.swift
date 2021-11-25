//
//  AddressCacheStoreError.swift
//  MullvadVPN
//
//  Created by pronebird on 08/12/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension AddressCache {
    enum StoreError: ChainedError {
        /// Failure to read address cache.
        case readCache(Swift.Error)

        /// Failure to read address cache from application bundle.
        case readCacheFromBundle(Swift.Error)

        /// Failure to write address cache.
        case writeCache(Swift.Error)

        /// Failure to decode address cache.
        case decodeCache(Swift.Error)

        /// Failure to encode address cache.
        case encodeCache(Swift.Error)

        /// Failure to decode address cache from application bundle.
        case decodeCacheFromBundle(Swift.Error)

        /// Failure to update endpoints with empty address list.
        case emptyAddressList

        var errorDescription: String? {
            switch self {
            case .readCache(_):
                return "Cannot read address cache"
            case .readCacheFromBundle(_):
                return "Cannot read address cache from application bundle"
            case .writeCache(_):
                return "Cannot write address cache"
            case .decodeCache(_):
                return "Cannot decode address cache"
            case .encodeCache(_):
                return "Cannot encode address cache"
            case .decodeCacheFromBundle(_):
                return "Cannot decode address cache from application bundle"
            case .emptyAddressList:
                return "Cannot update endpoints with empty address list"
            }
        }
    }
}
