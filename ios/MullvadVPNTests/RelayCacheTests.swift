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

final class RelayCacheTests: CachedTests {
    override class var cacheFileName: String { RelayCache.cacheFileName }

    func testReadReadsFromCache() throws {
        let didReadFromCache = expectation(description: "Cache was read")
        cacheFilePresenter.onReaderAction = {
            didReadFromCache.fulfill()
        }

        try withCachefolders { cacheDirectory, cacheFileURL in
            try prepopulateCache(at: cacheFileURL, fixedDate: .distantPast)

            let cache = RelayCache(cacheFolder: cacheDirectory)
            let relays = try cache.read()

            XCTAssertEqual(relays.updatedAt, .distantPast)
        }

        waitForExpectations(timeout: defaultExpectationTimeout)
    }

    func testWriteWritesToCache() throws {
        let didWriteToCache = expectation(description: "Cache was written to")
        cacheFilePresenter.onWriterAction = {
            didWriteToCache.fulfill()
        }

        try withCachefolders { cacheDirectory, cacheFileURL in
            let cache = RelayCache(cacheFolder: cacheDirectory)
            try cache.write(record: CachedRelays(relays: .empty, updatedAt: .distantPast))

            let cachedContent = try Data(contentsOf: cacheFileURL)
            let cachedRelays = try JSONDecoder().decode(CachedRelays.self, from: cachedContent)

            XCTAssertEqual(cachedRelays.updatedAt, .distantPast)
        }

        waitForExpectations(timeout: defaultExpectationTimeout)
    }
}

extension RelayCacheTests {
    func prepopulateCache(at cacheFileURL: URL, fixedDate: Date = .init()) throws {
        let prepopulatedCache = CachedRelays(relays: .empty, updatedAt: fixedDate)
        let encodedCache = try JSONEncoder().encode(prepopulatedCache)
        try encodedCache.write(to: cacheFileURL)
    }
}
