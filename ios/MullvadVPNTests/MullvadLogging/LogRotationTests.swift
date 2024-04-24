//
//  LogRotationTests.swift
//  MullvadVPNTests
//
//  Created by Jon Petersson on 2024-04-12.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadLogging
import XCTest

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

    func testRotatingActiveLogWhenSizeLimitIsExceeded() throws {
        let logName = "test.log"
        let logPath = directoryPath.appendingPathComponent(logName)

        let totalLogSizeLimit = 200
        let totalLogTestSize = 645
        let logChunkSize = 20

        let expectedLogCount = Int(ceil(Double(totalLogTestSize) / Double(totalLogSizeLimit)))
        let writeOperationCount = Int(ceil(Double(totalLogTestSize) / Double(logChunkSize)))

        let stream = LogFileOutputStream(fileURL: logPath, header: "", fileSizeLimit: UInt64(totalLogSizeLimit))
        for _ in 0 ..< writeOperationCount {
            stream.write(stringOfSize(logChunkSize))

            // Without sync between every write the test fails on Github.
            sync()
        }

        let actualLogCount = try fileManager.contentsOfDirectory(atPath: directoryPath.relativePath).count
        XCTAssertEqual(expectedLogCount, actualLogCount)

        for index in 0 ..< actualLogCount {
            var expectedFileName = logName

            if index != 0 {
                // Rotated log filenames start at "_2".
                expectedFileName = expectedFileName.replacingOccurrences(of: ".log", with: "_\(index + 1).log")
            }

            let logExists = fileManager.fileExists(
                atPath: directoryPath
                    .appendingPathComponent(expectedFileName)
                    .relativePath
            )
            XCTAssertTrue(logExists)
        }
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
        (0 ..< size).map { "\($0 % 10)" }.joined(separator: "")
    }

    private func writeDataToDisk(path: URL, fileSize: Int) throws {
        let data = Data((0 ..< fileSize).map { UInt8($0 & 0xff) })
        try data.write(to: path)
    }
}
