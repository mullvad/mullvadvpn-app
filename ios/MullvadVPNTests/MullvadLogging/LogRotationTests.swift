// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import XCTest

@testable import MullvadLogging

final class LogRotationTests: XCTestCase {
    let fileManager = FileManager.default
    var directoryPath: URL!

    override func setUpWithError() throws {
        directoryPath = FileManager.default.temporaryDirectory
            .appendingPathComponent("LogRotationTests", isDirectory: true)

        try fileManager.createDirectory(
            at: directoryPath,
            withIntermediateDirectories: true
        )
    }

    override func tearDownWithError() throws {
        try fileManager.removeItem(at: directoryPath)
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
            try writeDataToDisk(path: logPath, fileSize: 1000)
        }

        try LogRotation.rotateLogs(
            logDirectory: directoryPath,
            options: LogRotation.Options(
                storageSizeLimit: 5000,
                oldestAllowedDate: .distantPast
            )
        )
        var logFileCount = try fileManager.contentsOfDirectory(atPath: directoryPath.relativePath).count
        XCTAssertEqual(logFileCount, 5)

        try LogRotation.rotateLogs(
            logDirectory: directoryPath,
            options: LogRotation.Options(
                storageSizeLimit: 3999,
                oldestAllowedDate: .distantPast
            )
        )
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
            try writeDataToDisk(path: logPath, fileSize: 1000)
        }

        let oldestDateAllowedForSecondBatch = Date()
        try secondBatchOflogPaths.forEach { logPath in
            try writeDataToDisk(path: logPath, fileSize: 1000)
        }

        try LogRotation.rotateLogs(
            logDirectory: directoryPath,
            options: LogRotation.Options(storageSizeLimit: .max, oldestAllowedDate: oldestDateAllowedForFirstBatch)
        )
        var logFileCount = try fileManager.contentsOfDirectory(atPath: directoryPath.relativePath).count
        XCTAssertEqual(logFileCount, 5)

        try LogRotation.rotateLogs(
            logDirectory: directoryPath,
            options: LogRotation.Options(storageSizeLimit: .max, oldestAllowedDate: oldestDateAllowedForSecondBatch)
        )
        logFileCount = try fileManager.contentsOfDirectory(atPath: directoryPath.relativePath).count
        XCTAssertEqual(logFileCount, 2)
    }
}

extension LogRotationTests {
    private func stringOfSize(_ size: Int) -> String {
        (0..<size).map { "\($0 % 10)" }.joined(separator: "")
    }

    private func writeDataToDisk(path: URL, fileSize: Int) throws {
        let data = Data((0..<fileSize).map { UInt8($0 & 0xff) })
        try data.write(to: path)
    }
}
