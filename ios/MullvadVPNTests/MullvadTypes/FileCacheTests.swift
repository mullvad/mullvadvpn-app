//
//  FileCacheTests.swift
//  MullvadVPNTests
//
//  Created by pronebird on 13/06/2023.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
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
}
