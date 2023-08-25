//
//  MemoryCache.swift
//  MullvadRESTTests
//
//  Created by pronebird on 25/08/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
@testable import MullvadREST
import MullvadTypes

/// Mock implementation of a static memory cache passed to `AddressCache`.
/// Since we don't do any actual networking in tests, the IP endpoint returned from cache is not important.
struct MemoryCache: FileCacheProtocol {
    func read() throws -> REST.StoredAddressCache {
        return .init(updatedAt: .distantFuture, endpoint: .ipv4(IPv4Endpoint(ip: .loopback, port: 80)))
    }

    func write(_ content: REST.StoredAddressCache) throws {}
}
