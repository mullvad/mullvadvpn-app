//
//  CachedTests.swift
//  MullvadVPNTests
//
//  Created by Marco Nikic on 2023-06-02.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import MullvadREST
import XCTest

class CachedTests: XCTestCase {
    static var testsCacheDirectory: URL!
    var cacheFilePresenter: TestsCacheFilePresenter!
    let defaultExpectationTimeout = REST.Duration.milliseconds(200).timeInterval

    open class var cacheFileName: String {
        XCTFail("Do not use this class directly, inherit from it instead")
        return ""
    }

    override class func setUp() {
        super.setUp()
        let temporaryDirectory = FileManager.default.temporaryDirectory
        testsCacheDirectory = temporaryDirectory.appendingPathComponent("\(self)")
    }

    override func setUpWithError() throws {
        try super.setUpWithError()
        let cacheFileURL = Self.testsCacheDirectory.appendingPathComponent(Self.cacheFileName)
        cacheFilePresenter = TestsCacheFilePresenter(presentedItemURL: cacheFileURL)
        NSFileCoordinator.addFilePresenter(cacheFilePresenter)
    }

    override func tearDownWithError() throws {
        NSFileCoordinator.removeFilePresenter(cacheFilePresenter)
        try super.tearDownWithError()
    }
}

extension CachedTests {
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
}
