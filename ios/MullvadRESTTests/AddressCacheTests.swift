//
//  AddressCacheTests.swift
//  MullvadRESTTests
//
//  Created by Marco Nikic on 2023-05-05.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadREST
import MullvadTypes
import XCTest

final class AddressCacheTests: XCTestCase {
    static var testsCacheDirectory: URL!
    var apiEndpoint: AnyIPEndpoint!
    var cacheFilePresenter: AddressCacheFilePresenter!
    let defaultExpectationTimeout = REST.Duration.milliseconds(200).timeInterval

    // MARK: Tests Setup

    override class func setUp() {
        super.setUp()
        let temporaryDirectory = FileManager.default.temporaryDirectory
        testsCacheDirectory = temporaryDirectory.appendingPathComponent("AddressCacheTests")
    }

    override func setUpWithError() throws {
        try super.setUpWithError()
        apiEndpoint = try XCTUnwrap(AnyIPEndpoint(string: "127.0.0.1:80"))
        let cacheFileURL = Self.testsCacheDirectory.appendingPathComponent(REST.AddressCache.cacheFileName)
        cacheFilePresenter = AddressCacheFilePresenter(presentedItemURL: cacheFileURL)
        NSFileCoordinator.addFilePresenter(cacheFilePresenter)
    }

    override func tearDownWithError() throws {
        NSFileCoordinator.removeFilePresenter(cacheFilePresenter)
        try super.tearDownWithError()
    }

    // MARK: -

    // MARK: Tests

    func testAddressCacheHasDefaultEndpoint() {
        let cache = REST.AddressCache(canWriteToCache: false, cacheFolder: Self.testsCacheDirectory)
        XCTAssertEqual(cache.getCurrentEndpoint(), REST.defaultAPIEndpoint)
    }

    func testSetEndpoints() throws {
        let cache = REST.AddressCache(canWriteToCache: false, cacheFolder: Self.testsCacheDirectory)

        cache.setEndpoints([apiEndpoint])
        XCTAssertEqual(cache.getCurrentEndpoint(), apiEndpoint)
    }

    func testSetEndpointsUpdatesDateWhenSettingSameAddress() throws {
        let cache = REST.AddressCache(canWriteToCache: false, cacheFolder: Self.testsCacheDirectory)
        cache.setEndpoints([apiEndpoint])

        let dateBeforeSettingEndpoint = Date()
        cache.setEndpoints([apiEndpoint])
        let dateAfterSettingEndpoint = Date()

        let dateIntervalRange = dateBeforeSettingEndpoint ... dateAfterSettingEndpoint
        XCTAssertTrue(dateIntervalRange.contains(cache.getLastUpdateDate()))
    }

    func testSetEndpointsDoesNotDoAnythingIfSettingEmptyEndpoints() throws {
        let didNotWriteToCache = expectation(description: "Did not write to cache")
        didNotWriteToCache.isInverted = true

        cacheFilePresenter.onWriterAction = {
            didNotWriteToCache.fulfill()
        }

        try withCachefolders { cacheDirectory, _ in
            let cache = REST.AddressCache(canWriteToCache: true, cacheFolder: cacheDirectory)
            cache.setEndpoints([])
        }

        waitForExpectations(timeout: defaultExpectationTimeout)
    }

    func testSetEndpointsOnlyAcceptsTheFirstEndpoint() throws {
        let ipAddresses = (1 ... 10)
            .map { "\($0).\($0).\($0).\($0):80" }
            .compactMap { AnyIPEndpoint(string: $0) }

        let firstIPEndpoint = try XCTUnwrap(ipAddresses.first)

        try withCachefolders { cacheDirectory, cacheFileURL in
            let cache = REST.AddressCache(canWriteToCache: true, cacheFolder: cacheDirectory)
            cache.setEndpoints(ipAddresses)

            let cachedContent = try Data(contentsOf: cacheFileURL)
            let cachedAddresses = try JSONDecoder().decode(REST.CachedAddresses.self, from: cachedContent)

            XCTAssertEqual(cachedAddresses.endpoints.count, 1)
            XCTAssertEqual(cache.getCurrentEndpoint(), firstIPEndpoint)
        }
    }

    func testCacheReadsFromCachedFileAtInit() throws {
        let didReadFromCache = expectation(description: "Cache was read")
        cacheFilePresenter.onReaderAction = {
            didReadFromCache.fulfill()
        }

        try withCachefolders { cacheDirectory, cacheFileURL in
            let fixedDate = Date()
            try prepopulateCache(at: cacheFileURL, fixedDate: fixedDate, with: [apiEndpoint])
            let cache = REST.AddressCache(canWriteToCache: true, cacheFolder: cacheDirectory)

            XCTAssertEqual(cache.getCurrentEndpoint(), apiEndpoint)
            XCTAssertEqual(cache.getLastUpdateDate(), fixedDate)
        }

        waitForExpectations(timeout: defaultExpectationTimeout)
    }

    func testCacheWritesToDiskWhenSettingNewEndpoints() throws {
        let didWriteToCache = expectation(description: "Cache was written to")
        cacheFilePresenter.onWriterAction = {
            didWriteToCache.fulfill()
        }

        try withCachefolders { cacheDirectory, cacheFileURL in

            let cache = REST.AddressCache(canWriteToCache: true, cacheFolder: cacheDirectory)
            cache.setEndpoints([apiEndpoint])
            let cachedContent = try Data(contentsOf: cacheFileURL)
            let cachedAddresses = try JSONDecoder().decode(REST.CachedAddresses.self, from: cachedContent)
            let cachedAddress = try XCTUnwrap(cachedAddresses.endpoints.first)

            XCTAssertEqual(cachedAddress, cache.getCurrentEndpoint())
            XCTAssertEqual(cachedAddresses.updatedAt, cache.getLastUpdateDate())
        }

        waitForExpectations(timeout: defaultExpectationTimeout)
    }

    func testGetCurrentEndpointReadsFromCacheWhenReadOnly() throws {
        let didReadFromCache = expectation(description: "Cache was read")
        // Cache will be read from twice. Once during init, once when getting current endpoint
        didReadFromCache.expectedFulfillmentCount = 2
        cacheFilePresenter.onReaderAction = {
            didReadFromCache.fulfill()
        }

        try withCachefolders { cacheDirectory, cacheFileURL in
            let cache = REST.AddressCache(canWriteToCache: false, cacheFolder: cacheDirectory)
            try prepopulateCache(at: cacheFileURL, with: [apiEndpoint])

            XCTAssertEqual(cache.getCurrentEndpoint(), apiEndpoint)
        }

        waitForExpectations(timeout: defaultExpectationTimeout)
    }

    func testGetCurrentEndpointHasDefaultEndpointIfCacheIsEmpty() throws {
        let didReadFromCache = expectation(description: "Cache was read")
        // Cache will be read from twice. Once during init, once when getting current endpoint
        didReadFromCache.expectedFulfillmentCount = 2
        cacheFilePresenter.onReaderAction = {
            didReadFromCache.fulfill()
        }

        try withCachefolders { cacheDirectory, cacheFileURL in
            try prepopulateCache(at: cacheFileURL, with: [])

            let cache = REST.AddressCache(canWriteToCache: false, cacheFolder: cacheDirectory)
            XCTAssertEqual(cache.getCurrentEndpoint(), REST.defaultAPIEndpoint)
        }

        waitForExpectations(timeout: defaultExpectationTimeout)
    }
}

// MARK: -

extension AddressCacheTests {
    /// Prepares a cache folder that is expected to be present during the `runTest` closure
    /// - Parameter runTest: A closure that expects a `cacheDirectory` encapsulating `cacheFileURL` to be present when
    /// it runs
    func withCachefolders(_ runTest: (_ cacheDirectory: URL, _ cacheFileURL: URL) throws -> Void) throws {
        let cacheFileURL = try XCTUnwrap(cacheFilePresenter.presentedItemURL)
        let fileManager = FileManager.default
        let cacheDirectory = try XCTUnwrap(Self.testsCacheDirectory)
        try fileManager.createDirectory(at: cacheDirectory, withIntermediateDirectories: true)

        try runTest(cacheDirectory, cacheFileURL)

        try fileManager.removeItem(at: cacheDirectory)
    }

    /// Populates a JSON cache file containing a `Date` and `[AnyIPEndpoint]`
    ///
    /// - Parameters:
    ///   - cacheFileURL: The cache file destination
    ///   - fixedDate: The `Date` the cache file was written to
    ///   - endpoints: A list of `AnyIPEndpoint` to write in the cache
    func prepopulateCache(at cacheFileURL: URL, fixedDate: Date = Date(), with endpoints: [AnyIPEndpoint]) throws {
        let prepopulatedCache = REST.CachedAddresses(updatedAt: fixedDate, endpoints: endpoints)
        let encodedCache = try JSONEncoder().encode(prepopulatedCache)
        try encodedCache.write(to: cacheFileURL)
    }
}

class AddressCacheFilePresenter: NSObject, NSFilePresenter {
    var presentedItemURL: URL?
    let operationQueue: OperationQueue
    let dispatchQueue = DispatchQueue(label: "com.MullvadVPN.AddressCacheTests")
    var presentedItemOperationQueue: OperationQueue { operationQueue }

    var onReaderAction: (() -> Void)?
    var onWriterAction: (() -> Void)?

    init(presentedItemURL: URL) {
        operationQueue = OperationQueue()
        self.presentedItemURL = presentedItemURL
        operationQueue.underlyingQueue = dispatchQueue
    }

    func relinquishPresentedItem(toReader reader: @escaping ((() -> Void)?) -> Void) {
        print(#function)
        onReaderAction?()
        reader(nil)
    }

    func relinquishPresentedItem(toWriter writer: @escaping ((() -> Void)?) -> Void) {
        print(#function)
        onWriterAction?()
        writer(nil)
    }
}
