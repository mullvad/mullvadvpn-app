// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

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
