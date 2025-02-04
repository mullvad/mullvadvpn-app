//
//  RelayCacheTests.swift
//  MullvadVPNTests
//
//  Created by Marco Nikic on 2023-06-02.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadREST
import XCTest

final class RelayCacheTests: XCTestCase {
    func testReadCache() throws {
        let fileCache = MockFileCache(
            initialState: .exists(try StoredRelays(rawData: try .mock(), updatedAt: .distantPast))
        )
        let cache = RelayCache(fileCache: fileCache)
        let relays = try XCTUnwrap(cache.read())

        XCTAssertEqual(fileCache.getState(), .exists(relays))
    }

    func testWriteCache() throws {
        let fileCache = MockFileCache(
            initialState: .exists(try StoredRelays(rawData: try .mock(), updatedAt: .distantPast))
        )
        let cache = RelayCache(fileCache: fileCache)
        let newCachedRelays = try StoredRelays(rawData: try .mock(), updatedAt: Date())

        try cache.write(record: newCachedRelays)
        XCTAssertEqual(fileCache.getState(), .exists(newCachedRelays))
    }

    func testReadPrebundledRelaysWhenNoCacheIsStored() throws {
        let fileCache = MockFileCache<StoredRelays>(initialState: .fileNotFound)
        let cache = RelayCache(fileCache: fileCache)

        XCTAssertNoThrow(try cache.read())
    }
}

extension REST.ServerRelaysResponse {
    static func mock(
        serverRelays: [REST.ServerRelay] = [],
        bridgeRelays: [REST.BridgeRelay] = []
    ) -> Self {
        REST.ServerRelaysResponse(
            locations: [:],
            wireguard: REST.ServerWireguardTunnels(
                ipv4Gateway: .loopback,
                ipv6Gateway: .loopback,
                portRanges: [],
                relays: serverRelays,
                shadowsocksPortRanges: []
            ),
            bridge: REST.ServerBridges(shadowsocks: [], relays: bridgeRelays)
        )
    }
}

extension Data {
    static func mock(
        serverRelays: [REST.ServerRelay] = [],
        bridgeRelays: [REST.BridgeRelay] = []
    ) throws -> Data {
        let relays = REST.ServerRelaysResponse.mock(serverRelays: serverRelays, bridgeRelays: bridgeRelays)
        return try REST.Coding.makeJSONEncoder().encode(relays)
    }
}
