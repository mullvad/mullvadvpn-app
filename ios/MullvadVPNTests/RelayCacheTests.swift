//
//  RelayCacheTests.swift
//  MullvadVPNTests
//
//  Created by Marco Nikic on 2023-06-02.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadREST
import MullvadTransport
@testable import RelayCache
import XCTest

final class RelayCacheTests: XCTestCase {
    func testCanReadCache() throws {
        let fileCache = MockFileCache(
            initialState: .exists(CachedRelays(relays: .mock(), updatedAt: .distantPast))
        )
        let cache = RelayCache(fileCache: fileCache)
        let relays = try XCTUnwrap(cache.read())

        XCTAssertEqual(fileCache.getState(), .exists(relays))
    }

    func testCanWriteCache() throws {
        let fileCache = MockFileCache(
            initialState: .exists(CachedRelays(relays: .mock(), updatedAt: .distantPast))
        )
        let cache = RelayCache(fileCache: fileCache)
        let newCachedRelays = CachedRelays(relays: .mock(), updatedAt: Date())

        try cache.write(record: newCachedRelays)
        XCTAssertEqual(fileCache.getState(), .exists(newCachedRelays))
    }

    func testCanReadPrebundledRelaysWhenNoCacheIsStored() throws {
        let fileCache = MockFileCache<CachedRelays>(initialState: .fileNotFound)
        let cache = RelayCache(fileCache: fileCache)

        XCTAssertNoThrow(try cache.read())
    }
}

private extension REST.ServerRelaysResponse {
    static func mock() -> Self {
        REST.ServerRelaysResponse(
            locations: [:],
            wireguard: REST.ServerWireguardTunnels(
                ipv4Gateway: .loopback,
                ipv6Gateway: .loopback,
                portRanges: [],
                relays: []
            ),
            bridge: REST.ServerBridges(shadowsocks: [], relays: [])
        )
    }
}
