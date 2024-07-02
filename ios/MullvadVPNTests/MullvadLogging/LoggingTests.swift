//
//  LoggingTests.swift
//  MullvadVPNTests
//
//  Created by Emils on 04/04/2024.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
@testable import MullvadLogging
import XCTest

class MullvadLoggingTests: XCTestCase {
    let fileManager = FileManager.default
    var directoryPath: URL!

    override func setUpWithError() throws {
        directoryPath = FileManager.default.temporaryDirectory.appendingPathComponent("LoggingTests", isDirectory: true)

        try fileManager.createDirectory(
            at: directoryPath,
            withIntermediateDirectories: true
        )
    }

    override func tearDownWithError() throws {
        try fileManager.removeItem(at: directoryPath)
    }

    func testLogFileOutputStreamWritesHeader() throws {
        let headerText = "This is a header"
        let logMessage = "And this is a log message\n"
        let fileURL = directoryPath.appendingPathComponent(UUID().uuidString)
        let stream = LogFileOutputStream(fileURL: fileURL, header: headerText)
        stream.write(logMessage)
        sync()

        let contents = try XCTUnwrap(String(contentsOf: fileURL))
        XCTAssertEqual(contents, "\(headerText)\n\(logMessage)")
    }

    func testLogHeader() throws {
        let expectedHeader = "Header of a log file"

        var builder = LoggerBuilder(header: expectedHeader)
        let fileURL = directoryPath.appendingPathComponent(UUID().uuidString)
        builder.addFileOutput(fileURL: fileURL)

        builder.install()

        Logger(label: "test").info(":-P")

        sync()

        let contents = try XCTUnwrap(String(contentsOf: fileURL))

        XCTAssert(contents.hasPrefix(expectedHeader))
    }

    func testGettingLogFilesByApplicationTarget() async throws {
        let mainTargetLog = ApplicationConfiguration.newLogFileURL(for: .mainApp, in: directoryPath)
        let packetTunnelTargetLog = ApplicationConfiguration.newLogFileURL(for: .packetTunnel, in: directoryPath)

        let logPaths = [
            directoryPath.appendingPathComponent("test1.log"),
            directoryPath.appendingPathComponent("test2.log"),
            mainTargetLog,
            packetTunnelTargetLog,
        ]

        logPaths.forEach { url in
            let stream = LogFileOutputStream(fileURL: url, header: "")
            stream.write("test")
            sync()
        }

        var urls = ApplicationConfiguration.logFileURLs(for: .mainApp, in: directoryPath)
        XCTAssertEqual(urls.count, 1)
        XCTAssertEqual(urls.first, mainTargetLog)

        urls = ApplicationConfiguration.logFileURLs(for: .packetTunnel, in: directoryPath)
        XCTAssertEqual(urls.count, 1)
        XCTAssertEqual(urls.first, packetTunnelTargetLog)
    }
}
