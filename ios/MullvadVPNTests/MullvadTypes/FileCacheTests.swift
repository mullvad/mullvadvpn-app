//
//  FileCacheTests.swift
//  MullvadVPNTests
//
//  Created by pronebird on 13/06/2023.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import XCTest

@testable import MullvadTypes

class FileCacheTests: XCTestCase {
    var testFileURL: URL!

    override func setUp() {
        testFileURL = FileManager.default.temporaryDirectory
            .appendingPathComponent("FileCacheTest-\(UUID().uuidString)", isDirectory: false)
    }

    override func tearDown() {
        try? FileManager.default.removeItem(at: testFileURL)
    }

    func testRead() throws {
        let stringData = UUID().uuidString
        try JSONEncoder().encode(stringData).write(to: testFileURL)

        let fileCache = FileCache<String>(fileURL: testFileURL)
        XCTAssertEqual(try fileCache.read(), stringData)
    }

    func testWrite() throws {
        let fileCache = FileCache<String>(fileURL: testFileURL)

        let stringData = UUID().uuidString
        let serializedData = try JSONEncoder().encode(stringData)

        try fileCache.write(stringData)

        XCTAssertEqual(try Data(contentsOf: testFileURL), serializedData)
    }

    // MARK: - Cache behavior

    func testReadReturnsCachedContentOnSubsequentCalls() throws {
        let value = "cached-value"
        try JSONEncoder().encode(value).write(to: testFileURL)

        let fileCache = FileCache<String>(fileURL: testFileURL)
        let first = try fileCache.read()
        let second = try fileCache.read()

        XCTAssertEqual(first, value)
        XCTAssertEqual(second, value)
    }

    func testWriteUpdatesCachedContent() throws {
        let fileCache = FileCache<String>(fileURL: testFileURL)

        try fileCache.write("first")
        XCTAssertEqual(try fileCache.read(), "first")

        try fileCache.write("second")
        XCTAssertEqual(try fileCache.read(), "second")
    }

    func testClearInvalidatesCache() throws {
        let fileCache = FileCache<String>(fileURL: testFileURL)

        try fileCache.write("value")
        XCTAssertEqual(try fileCache.read(), "value")

        try fileCache.clear()

        XCTAssertThrowsError(try fileCache.read())
    }

    // MARK: - Thundering herd

    /// Spawn many concurrent readers on a single FileCache instance to verify there is no deadlock
    /// and all readers return the correct value.
    func testThunderingHerdReads() throws {
        let value = "herd-read-value"
        try JSONEncoder().encode(value).write(to: testFileURL)

        let fileCache = FileCache<String>(fileURL: testFileURL)
        let iterations = 200

        DispatchQueue.concurrentPerform(iterations: iterations) { _ in
            do {
                let result = try fileCache.read()
                XCTAssertEqual(result, value)
            } catch {
                XCTFail("Concurrent read failed: \(error)")
            }
        }
    }

    /// Spawn many concurrent writers that each write a unique value, then verify the file contains
    /// one of the written values and reading back returns the same value.
    func testThunderingHerdWrites() throws {
        let fileCache = FileCache<String>(fileURL: testFileURL)
        let iterations = 200

        DispatchQueue.concurrentPerform(iterations: iterations) { i in
            do {
                try fileCache.write("value-\(i)")
            } catch {
                XCTFail("Concurrent write failed: \(error)")
            }
        }

        // The last writer wins; just verify we can read back consistently.
        let result = try fileCache.read()
        XCTAssertTrue(result.hasPrefix("value-"), "Expected one of the written values, got: \(result)")
    }

    /// Interleave reads and writes from many threads to check for deadlocks and data races.
    func testThunderingHerdMixedReadsAndWrites() throws {
        try JSONEncoder().encode("initial").write(to: testFileURL)

        let fileCache = FileCache<String>(fileURL: testFileURL)
        let iterations = 200

        DispatchQueue.concurrentPerform(iterations: iterations) { i in
            do {
                if i.isMultiple(of: 3) {
                    try fileCache.write("mixed-\(i)")
                } else {
                    _ = try fileCache.read()
                }
            } catch {
                XCTFail("Mixed concurrent operation failed at iteration \(i): \(error)")
            }
        }

        let result = try fileCache.read()
        XCTAssertFalse(result.isEmpty)
    }

    // MARK: - Deadlock smoke tests

    /// Rapidly alternate write-then-read on many threads to provoke coordinator / cache-queue ordering issues.
    func testWriteThenReadDoesNotDeadlock() throws {
        let fileCache = FileCache<String>(fileURL: testFileURL)
        let iterations = 100

        DispatchQueue.concurrentPerform(iterations: iterations) { i in
            do {
                try fileCache.write("wtr-\(i)")
                let result = try fileCache.read()
                XCTAssertTrue(result.hasPrefix("wtr-"))
            } catch {
                XCTFail("Write-then-read failed at iteration \(i): \(error)")
            }
        }
    }

    /// Rapidly alternate write-then-clear on many threads to exercise the forDeleting / forReplacing coordination paths.
    func testConcurrentWriteAndClear() throws {
        let fileCache = FileCache<String>(fileURL: testFileURL)
        let iterations = 100

        DispatchQueue.concurrentPerform(iterations: iterations) { i in
            if i.isMultiple(of: 2) {
                try? fileCache.write("clear-\(i)")
            } else {
                try? fileCache.clear()
            }
        }

        // After the storm, write a known value and verify consistency.
        try fileCache.write("final")
        XCTAssertEqual(try fileCache.read(), "final")
    }

    /// Simulate the iOS 17 scenario where `presentedItemDidChange` fires while coordinated operations
    /// are in flight by calling it manually from many threads alongside reads and writes.
    func testPresenterCallbacksDuringConcurrentAccess() throws {
        try JSONEncoder().encode("presenter-initial").write(to: testFileURL)

        let fileCache = FileCache<String>(fileURL: testFileURL)
        let iterations = 200

        DispatchQueue.concurrentPerform(iterations: iterations) { i in
            switch i % 4 {
            case 0:
                try? fileCache.write("presenter-\(i)")
            case 1:
                _ = try? fileCache.read()
            case 2:
                // Simulate the iOS 17 spurious presenter callback.
                fileCache.presentedItemDidChange()
            default:
                fileCache.accommodatePresentedItemDeletion { _ in }
            }
        }

        // Stabilize: write a known value and confirm round-trip.
        try fileCache.write("after-presenter-storm")
        XCTAssertEqual(try fileCache.read(), "after-presenter-storm")
    }

    /// Use two independent FileCache instances pointing at the same file to exercise cross-instance
    /// file coordination and verify no deadlock occurs.
    func testTwoInstancesSameFile() throws {
        try JSONEncoder().encode("shared-initial").write(to: testFileURL)

        let cacheA = FileCache<String>(fileURL: testFileURL)
        let cacheB = FileCache<String>(fileURL: testFileURL)
        let iterations = 100

        DispatchQueue.concurrentPerform(iterations: iterations) { i in
            do {
                if i.isMultiple(of: 2) {
                    try cacheA.write("A-\(i)")
                } else {
                    try cacheB.write("B-\(i)")
                }
            } catch {
                XCTFail("Two-instance write failed at iteration \(i): \(error)")
            }
        }

        // Both caches should be able to read a consistent value.
        let resultA = try cacheA.read()
        let resultB = try cacheB.read()
        XCTAssertFalse(resultA.isEmpty)
        XCTAssertFalse(resultB.isEmpty)
    }

    /// Deadlock smoke test with a timeout — if any coordinated operation deadlocks, the test will
    /// fail by exceeding the XCTest timeout rather than hanging the suite indefinitely.
    func testNoDeadlockUnderTimeout() throws {
        try JSONEncoder().encode("timeout-test").write(to: testFileURL)

        let fileCache = FileCache<String>(fileURL: testFileURL)
        let expectation = expectation(description: "All concurrent operations complete")
        let iterations = 300
        let group = DispatchGroup()

        for i in 0..<iterations {
            group.enter()
            DispatchQueue.global().async {
                defer { group.leave() }
                do {
                    switch i % 5 {
                    case 0: try fileCache.write("timeout-\(i)")
                    case 1: _ = try fileCache.read()
                    case 2: try fileCache.clear(); try fileCache.write("recovered-\(i)")
                    case 3: fileCache.presentedItemDidChange()
                    default: fileCache.accommodatePresentedItemDeletion { _ in }
                    }
                } catch {
                    // Errors from clear/read races are expected; only deadlocks matter here.
                }
            }
        }

        group.notify(queue: .main) {
            expectation.fulfill()
        }

        wait(for: [expectation], timeout: 30)
    }
}
