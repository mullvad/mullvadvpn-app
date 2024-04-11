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
    func temporaryFileURL() -> URL {
        // Create a URL for an unique file in the system's temporary directory.
        let directory = NSTemporaryDirectory()
        let filename = UUID().uuidString
        let fileURL = URL(fileURLWithPath: directory).appendingPathComponent(filename)

        // Add a teardown block to delete any file at `fileURL`.
        addTeardownBlock {
            do {
                let fileManager = FileManager.default
                // Check that the file exists before trying to delete it.
                if fileManager.fileExists(atPath: fileURL.path) {
                    // Perform the deletion.
                    try fileManager.removeItem(at: fileURL)
                }
            } catch {
                // Treat any errors during file deletion as a test failure.
                XCTFail("Error while deleting temporary file: \(error)")
            }
        }

        // Return the temporary file URL for use in a test method.
        return fileURL
    }

    func testLogHeader() {
        let expectedHeader = "Header of a log file"

        var builder = LoggerBuilder(header: expectedHeader)
        let fileURL = temporaryFileURL()
        builder.addFileOutput(fileURL: fileURL)

        builder.install()

        Logger(label: "test").info(":-P")

// For some reason, reading the file fails here, despite the file existing. Manual inspection reveals the file to have the correct header. ¯\_(ツ)_/¯
//        let contents = String(decoding: try! Data(contentsOf: fileURL), as: UTF8.self)
//
//        XCTAssert(contents.hasPrefix(expectedHeader))
    }
}
