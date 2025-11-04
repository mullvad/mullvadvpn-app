//
//  LoggingTests.swift
//  MullvadVPNTests
//
//  Created by Emils on 04/04/2024.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import XCTest

@testable import MullvadLogging

class LoggingTests: XCTestCase {
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
            stream.synchronize()
        }

        var urls = ApplicationConfiguration.logFileURLs(for: .mainApp, in: directoryPath)
        XCTAssertEqual(urls.count, 1)
        XCTAssertEqual(urls.first, mainTargetLog)

        urls = ApplicationConfiguration.logFileURLs(for: .packetTunnel, in: directoryPath)
        XCTAssertEqual(urls.count, 1)
        XCTAssertEqual(urls.first, packetTunnelTargetLog)
    }
}
