//
//  RelayCacheTests.swift
//  MullvadVPNTests
//
//  Created by Marco Nikic on 2023-06-02.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadREST
import XCTest

final class RelayCacheTests: XCTestCase {
    func testReadCache() throws {
        let fileCache = MockFileCache(
            initialState: .exists(CachedRelays(relays: .mock(), updatedAt: .distantPast))
        )
        let cache = RelayCache(fileCache: fileCache)
        let relays = try XCTUnwrap(cache.read())

        XCTAssertEqual(fileCache.getState(), .exists(relays))
    }

    func testWriteCache() throws {
        let fileCache = MockFileCache(
            initialState: .exists(CachedRelays(relays: .mock(), updatedAt: .distantPast))
        )
        let cache = RelayCache(fileCache: fileCache)
        let newCachedRelays = CachedRelays(relays: .mock(), updatedAt: Date())

        try cache.write(record: newCachedRelays)
        XCTAssertEqual(fileCache.getState(), .exists(newCachedRelays))
    }

    func testReadPrebundledRelaysWhenNoCacheIsStored() throws {
        let fileCache = MockFileCache<CachedRelays>(initialState: .fileNotFound)
        let cache = RelayCache(fileCache: fileCache)

        XCTAssertNoThrow(try cache.read())
    }
}

extension REST.ServerRelaysResponse {
    static func mock(
        serverRelays: [REST.ServerRelay] = [],
        brideRelays: [REST.BridgeRelay] = []
    ) -> Self {
        REST.ServerRelaysResponse(
            locations: [:],
            wireguard: REST.ServerWireguardTunnels(
                ipv4Gateway: .loopback,
                ipv6Gateway: .loopback,
                portRanges: [],
                relays: serverRelays
            ),
            bridge: REST.ServerBridges(shadowsocks: [], relays: brideRelays)
        )
    }
}
