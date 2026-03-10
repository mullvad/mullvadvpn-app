//
//  RelayCacheTests.swift
//  MullvadVPNTests
//
//  Created by Marco Nikic on 2023-06-02.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadMockData
import MullvadTypes
import XCTest

@testable import MullvadREST

final class RelayCacheTests: XCTestCase {
    func testReadCache() throws {
        let fileCache = MockFileCache(
            initialState: .exists(try StoredRelays(rawData: try .mock(), updatedAt: .distantPast))
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

    func testEmptyRelaysIsEmpty() {
        let emptyRelays = REST.ServerRelaysResponse.mock(serverRelays: [], bridgeRelays: [])
        XCTAssertTrue(emptyRelays.isEmpty)

        let cachedRelays = CachedRelays(etag: nil, relays: emptyRelays, updatedAt: Date())
        XCTAssertTrue(cachedRelays.isEmpty)
    }

    /// Proves that unknown relay list JSON fields survive a Codable round-trip through `FileCache`.
    func testRawDataPreservesUnknownFieldsThroughFileCacheRoundTrip() throws {
        let jsonWithUnknownField = try ServerRelaysResponseStubs.sampleRelaysJSONWithUnknownField()

        // Use real FileCache to exercise the Codable round-trip.
        let tempDir = FileManager.default.temporaryDirectory
            .appendingPathComponent(UUID().uuidString, isDirectory: true)
        try FileManager.default.createDirectory(at: tempDir, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: tempDir) }

        let fileCache = FileCache<StoredRelays>(
            fileURL: tempDir.appendingPathComponent("relays.json")
        )

        let stored = try StoredRelays(rawData: jsonWithUnknownField, updatedAt: Date())
        try fileCache.write(stored)

        // Force a fresh read from disk (new FileCache instance, no in-memory cache).
        let freshFileCache = FileCache<StoredRelays>(
            fileURL: tempDir.appendingPathComponent("relays.json")
        )
        let readBack = try freshFileCache.read()

        // rawData must survive the Codable round-trip byte-for-byte.
        XCTAssertEqual(readBack.rawData, jsonWithUnknownField)

        // The unknown field must still be present.
        let json = try JSONSerialization.jsonObject(with: readBack.rawData) as! [String: Any]
        XCTAssertNotNil(json["future_feature"], "Unknown field should survive the round-trip")

        // On main, the Codable encoding of StoredRelays included a redundant `relays` key.
        // Verify it is NOT present — only etag, rawData, and updatedAt should be encoded.
        let diskData = try Data(contentsOf: tempDir.appendingPathComponent("relays.json"))
        let diskJson = try JSONSerialization.jsonObject(with: diskData) as! [String: Any]
        XCTAssertNil(diskJson["relays"], "StoredRelays should not encode a redundant 'relays' key")
    }

    /// Constructing `StoredRelays` with invalid data should fail at init time
    /// because deserialization is performed eagerly.
    func testInitWithInvalidDataThrows() {
        let notARelayResponse = """
            {"something": "completely different"}
            """.data(using: .utf8)!

        XCTAssertThrowsError(try StoredRelays(rawData: notARelayResponse, updatedAt: Date()))
    }

    /// Verifies that `cachedRelays` returns the cached result without re-deserializing.
    /// The deserialization cache is populated at init time; subsequent accesses reuse it.
    func testCachedRelaysDeserializesOnlyOnce() throws {
        let rawData: Data = try .mock()
        let stored = try StoredRelays(rawData: rawData, updatedAt: Date())

        let first = try stored.cachedRelays
        let second = try stored.cachedRelays

        // Both accesses return the same deserialized result.
        XCTAssertEqual(first, second)
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
