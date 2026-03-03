//
//  RelayCacheTests.swift
//  MullvadVPNTests
//
//  Created by Marco Nikic on 2023-06-02.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadMockData
import XCTest

@testable import MullvadREST

final class RelayCacheTests: XCTestCase {
    func testReadCache() throws {
        let fileCache = MockFileCache(
            initialState: .exists(StoredRelays(rawData: try .mock(), updatedAt: .distantPast))
        )
        let cache = RelayCache(fileCache: fileCache)
        let relays = try XCTUnwrap(cache.read())

        if case let .exists(storedRelays) = fileCache.getState() {
            XCTAssertEqual(try storedRelays.cachedRelays, relays)
        } else {
            XCTFail("Expected existing state, got \(fileCache.getState())")
        }
    }

    func testWriteCache() throws {
        let fileCache = MockFileCache(
            initialState: .exists(StoredRelays(rawData: try .mock(), updatedAt: .distantPast))
        )
        let cache = RelayCache(fileCache: fileCache)
        let rawData: Data = try .mock()
        let newCachedRelays = StoredRelays(rawData: rawData, updatedAt: Date())

        try cache.write(record: newCachedRelays)
        XCTAssertEqual(fileCache.getState(), .exists(newCachedRelays))
    }

    func testReadPrebundledRelaysWhenNoCacheIsStored() throws {
        let fileCache = MockFileCache<StoredRelays>(initialState: .fileNotFound)
        let cache = RelayCache(fileCache: fileCache)

        XCTAssertNoThrow(try cache.read())
    }

    func testEmptyRelaysIsEmpty() {
        let emptyRelays = REST.ServerRelaysResponse.mock(serverRelays: [], bridgeRelays: [])
        XCTAssertTrue(emptyRelays.isEmpty)

        let cachedRelays = CachedRelays(etag: nil, relays: emptyRelays, updatedAt: Date())
        XCTAssertTrue(cachedRelays.isEmpty)
    }

    /// Proves that unknown JSON fields (e.g. a new feature the model doesn't know about)
    /// survive a write-then-read round-trip through `StoredRelays.rawData`.
    func testRawDataPreservesUnknownFields() throws {
        // JSON with an extra top-level key "future_feature" that ServerRelaysResponse doesn't model.
        let jsonWithUnknownField = """
            {
                "locations": {},
                "wireguard": {
                    "ipv4_gateway": "10.64.0.1",
                    "ipv6_gateway": "fc00:bbbb:bbbb:bb01::1",
                    "port_ranges": [],
                    "relays": [],
                    "shadowsocks_port_ranges": []
                },
                "bridge": {
                    "shadowsocks": [],
                    "relays": []
                },
                "future_feature": {
                    "key": "value",
                    "nested": [1, 2, 3]
                }
            }
            """.data(using: .utf8)!

        let fileCache = MockFileCache<StoredRelays>(initialState: .fileNotFound)
        let cache = RelayCache(fileCache: fileCache)

        let stored = StoredRelays(rawData: jsonWithUnknownField, updatedAt: Date())
        try cache.write(record: stored)

        // Read it back and verify rawData is byte-for-byte identical.
        let readBack = try fileCache.read()
        XCTAssertEqual(readBack.rawData, jsonWithUnknownField)

        // Double-check the unknown field is still present in the JSON.
        let json = try JSONSerialization.jsonObject(with: readBack.rawData) as! [String: Any]
        XCTAssertNotNil(json["future_feature"], "Unknown field should survive the round-trip")
    }

    func testNonEmptyRelaysIsNotEmpty() {
        let relays = ServerRelaysResponseStubs.sampleRelays
        XCTAssertFalse(relays.isEmpty)

        let cachedRelays = CachedRelays(etag: nil, relays: relays, updatedAt: Date())
        XCTAssertFalse(cachedRelays.isEmpty)
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
