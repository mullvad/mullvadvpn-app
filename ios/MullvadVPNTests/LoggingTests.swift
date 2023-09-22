//
//  LoggingTests.swift
//  MullvadVPNTests
//
//  Created by Emils on 04/04/2024.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import XCTest
@testable import MullvadLogging

class MullvadLoggingTests: XCTestCase {
    func testLogHeader() {
        let dummySig = "test-sgi";
        let testFileName = "test"
        let expectedHeader = "Header of a log file"
        
        var builder = LoggerBuilder()
        try! builder.addFileOutput(securityGroupIdentifier: dummySig, basename: testFileName)
        
        builder.install(header: expectedHeader)
        
        let logFileUrl = FileManager.default.containerURL(forSecurityApplicationGroupIdentifier: dummySig)!.appendingPathComponent("Logs", isDirectory: true).appendingPathComponent("\(testFileName).log", isDirectory: false)
        
        let contents = String(decoding: try! Data(contentsOf: logFileUrl), as: UTF8.self)
        
        XCTAssert(contents.hasPrefix(expectedHeader))
        XCTAssertEqual("\(expectedHeader)\n", contents)
    }
}
