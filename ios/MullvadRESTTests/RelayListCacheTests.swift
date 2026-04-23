//
//  RelayListCacheE2ETests.swift
//  MullvadRESTTests
//
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadMockData
import MullvadRustRuntime
import MullvadTypes
import Network
import XCTest

@testable import MullvadREST
@testable import MullvadSettings

/// End-to-end test that verifies relay list responses are stored verbatim in the cache,
/// preserving unknown JSON fields that `ServerRelaysResponse` doesn't model.
///
/// This test would fail on main where `StoredRelays` encoded a deserialized `relays`
/// property via Codable, losing any fields not declared in `ServerRelaysResponse`.
class RelayListCacheTests: XCTestCase {
    static let store = InMemorySettingsStore<SettingNotFound>()

    override func setUp() {
        super.setUp()
        SettingsManager.unitTestStore = Self.store
        RustLogging.initialize()
    }

    func testRelayListWithUnknownFieldsSurvivesFullPipeline() async throws {
        // Relay JSON based on sampleRelays with an extra unknown top-level field.
        let relayJSONData = try ServerRelaysResponseStubs.sampleRelaysJSONWithUnknownField()
        let relayJSON = String(data: relayJSONData, encoding: .utf8)!

        // 1. Mock the relay list endpoint with our JSON containing unknown fields.
        let mock = MullvadApiMock.get(
            path: "/app/v1/relays",
            responseCode: 200,
            responseData: relayJSON
        )
        let apiProxy = try makeApiProxy(port: mock.port)

        // 2. Fetch relays through the API proxy (exercises the full Rust FFI path).
        let result: Result<REST.ServerRelaysCacheResponse, Error> =
            await withCheckedContinuation { continuation in
                _ = apiProxy.getRelays(
                    etag: nil,
                    retryStrategy: .noRetry
                ) { result in
                    continuation.resume(returning: result)
                }
            }

        guard case let .newContent(etag, rawData) = try result.get() else {
            XCTFail("Expected .newContent response")
            return
        }

        // 3. Store through RelayCache backed by real FileCache (Codable round-trip).
        let tempDir = FileManager.default.temporaryDirectory
            .appendingPathComponent(UUID().uuidString, isDirectory: true)
        try FileManager.default.createDirectory(at: tempDir, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: tempDir) }

        let fileURL = tempDir.appendingPathComponent("relays.json")
        let fileCache = FileCache<StoredRelays>(fileURL: fileURL)
        let relayCache = RelayCache(fileCache: fileCache)

        let storedRelays = try StoredRelays(etag: etag, rawData: rawData, updatedAt: Date())
        try relayCache.write(record: storedRelays)

        // 4. Read back from a fresh FileCache (forces disk round-trip, no in-memory cache).
        let freshFileCache = FileCache<StoredRelays>(fileURL: fileURL)
        let readBack = try freshFileCache.read()

        // 5. Verify rawData is preserved byte-for-byte.
        XCTAssertEqual(readBack.rawData, rawData)

        // 6. Verify the unknown field survived the full pipeline.
        let json = try JSONSerialization.jsonObject(with: readBack.rawData) as! [String: Any]
        XCTAssertNotNil(
            json["future_feature"],
            "Unknown JSON field 'future_feature' should survive the full fetch→store→read pipeline"
        )

        // 7. Verify deserialization still works for known fields.
        let cachedRelays = try readBack.cachedRelays
        XCTAssertNotNil(cachedRelays)
    }

    // MARK: - Helpers

    private func makeApiProxy(port: UInt16) throws -> APIQuerying {
        let shadowsocksLoader = ShadowsocksLoaderStub(
            configuration: ShadowsocksConfiguration(
                address: .ipv4(.loopback),
                port: 1080,
                password: "123",
                cipher: "aes-128-cfb"
            )
        )

        let accessMethodsRepository = AccessMethodRepositoryStub.stub

        let context = try MullvadApiContext(
            host: "localhost",
            address: "\(IPv4Address.loopback.debugDescription):\(port)",
            domain: REST.encryptedDNSHostname,
            disableTls: true,
            shadowsocksProvider: shadowsocksLoader,
            accessMethodWrapper: initAccessMethodSettingsWrapper(methods: accessMethodsRepository.fetchAll()),
            accessMethodChangeListeners: []
        )

        return REST.MullvadAPIProxy(
            transportProvider: APITransportProvider(
                requestFactory: .init(
                    apiContext: context,
                    encoder: JSONEncoder()
                )
            ),
            dispatchQueue: .main,
            responseDecoder: REST.Coding.makeJSONDecoder()
        )
    }
}
