//
//  LogRotationTests.swift
//  MullvadVPNTests
//
//  Created by Jon Petersson on 2024-04-12.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import MullvadLogging
import XCTest

final class LogRotationTests: XCTestCase {
    let fileManager = FileManager.default
    let directoryPath = FileManager.default.temporaryDirectory.appendingPathComponent("LogRotationTests")

    override func setUpWithError() throws {
        try? fileManager.createDirectory(
            at: directoryPath,
            withIntermediateDirectories: false
        )
    }

    override func tearDownWithError() throws {
        try fileManager.removeItem(atPath: directoryPath.relativePath)
    }

    func testRotateLogsByStorageSizeLimit() throws {
        let logPaths = [
            directoryPath.appendingPathComponent("test1.log"),
            directoryPath.appendingPathComponent("test2.log"),
            directoryPath.appendingPathComponent("test3.log"),
            directoryPath.appendingPathComponent("test4.log"),
            directoryPath.appendingPathComponent("test5.log"),
        ]

        try logPaths.forEach { logPath in
            try writeDataToDisk(path: logPath, sizeInBytes: 1000)
        }

        try LogRotation.rotateLogs(logDirectory: directoryPath, options: LogRotation.Options(storageSizeLimit: 5000))
        var logFileCount = try fileManager.contentsOfDirectory(atPath: directoryPath.relativePath).count
        XCTAssertEqual(logFileCount, 5)

        try LogRotation.rotateLogs(logDirectory: directoryPath, options: LogRotation.Options(storageSizeLimit: 3999))
        logFileCount = try fileManager.contentsOfDirectory(atPath: directoryPath.relativePath).count
        XCTAssertEqual(logFileCount, 3)
    }

    func testRotateLogsByOldestAllowedDate() throws {
        let firstBatchOflogPaths = [
            directoryPath.appendingPathComponent("test1.log"),
            directoryPath.appendingPathComponent("test2.log"),
            directoryPath.appendingPathComponent("test3.log"),
        ]

        let secondBatchOflogPaths = [
            directoryPath.appendingPathComponent("test4.log"),
            directoryPath.appendingPathComponent("test5.log"),
        ]

        let oldestDateAllowedForFirstBatch = Date()
        try firstBatchOflogPaths.forEach { logPath in
            try writeDataToDisk(path: logPath, sizeInBytes: 1000)
        }

        let oldestDateAllowedForSecondBatch = Date()
        try secondBatchOflogPaths.forEach { logPath in
            try writeDataToDisk(path: logPath, sizeInBytes: 1000)
        }

        try LogRotation.rotateLogs(
            logDirectory: directoryPath,
            options: LogRotation.Options(oldestAllowedDate: oldestDateAllowedForFirstBatch)
        )
        var logFileCount = try fileManager.contentsOfDirectory(atPath: directoryPath.relativePath).count
        XCTAssertEqual(logFileCount, 5)

        try LogRotation.rotateLogs(
            logDirectory: directoryPath,
            options: LogRotation.Options(oldestAllowedDate: oldestDateAllowedForSecondBatch)
        )
        logFileCount = try fileManager.contentsOfDirectory(atPath: directoryPath.relativePath).count
        XCTAssertEqual(logFileCount, 2)
    }
}

extension LogRotationTests {
    private func writeDataToDisk(path: URL, sizeInBytes: Int) throws {
        let data = Data((0 ..< sizeInBytes).map { UInt8($0 & 0xff) })
        try data.write(to: path)
    }
}
