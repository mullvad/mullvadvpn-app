//
//  RelayCacheTests.swift
//  MullvadVPNTests
//
//  Created by Marco Nikic on 2023-06-02.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import MullvadREST
import MullvadTransport
@testable import RelayCache
import XCTest

final class RelayCacheTests: XCTestCase {
    func testReadReadsFromCache() throws {
        let fileCache = MockFileCache(
            initialState: .exists(CachedRelays(relays: .empty, updatedAt: .distantPast))
        )
        let cache = RelayCache(fileCache: fileCache)
        let relays = try XCTUnwrap(cache.read())

        XCTAssertEqual(fileCache.getState(), .exists(relays))
    }

    func testWriteWritesToCache() throws {
        let fileCache = MockFileCache(
            initialState: .exists(CachedRelays(relays: .empty, updatedAt: .distantPast))
        )
        let cache = RelayCache(fileCache: fileCache)
        let newCachedRelays = CachedRelays(relays: .empty, updatedAt: Date())

        try cache.write(record: newCachedRelays)
        XCTAssertEqual(fileCache.getState(), .exists(newCachedRelays))
    }
}
