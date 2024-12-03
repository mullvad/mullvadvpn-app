//
//  ConsolidatedApplicationLogTests.swift
//  MullvadVPNTests
//
//  Created by Mojgan on 2024-11-21.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import XCTest

class ConsolidatedApplicationLogTests: XCTestCase {
    var consolidatedLog: ConsolidatedApplicationLog!
    let mockRedactStrings = ["sensitive", "secret"]
    let mockSecurityGroupIdentifiers = ["group1", "group2"]
    var createdMockFiles: [URL] = []
    let kRedactedPlaceholder = "[REDACTED]"

    override func setUp() {
        super.setUp()
        consolidatedLog = ConsolidatedApplicationLog(
            redactCustomStrings: mockRedactStrings,
            redactContainerPathsForSecurityGroupIdentifiers: mockSecurityGroupIdentifiers,
            bufferSize: 65_536
        )
        createdMockFiles = []
    }

    override func tearDownWithError() throws {
        try super.tearDownWithError()
        consolidatedLog = nil
        // Remove all mock files created during tests
        for file in createdMockFiles {
            try FileManager.default.removeItem(at: file)
        }
        createdMockFiles = []
    }

    func testAddLogFiles() {
        var string = ""
        let expectation = self.expectation(description: "Log files added")
        let mockFile = createMockFile(content: content, fileName: "\(generateRandomName()).txt")

        consolidatedLog.addLogFiles(fileURLs: [mockFile]) {
            expectation.fulfill()
        }

        waitForExpectations(timeout: 1)
        consolidatedLog.write(to: &string)
        XCTAssertTrue(
            consolidatedLog.string.contains(string),
            "Log should contain the file content."
        )
    }

    func testAddError() {
        let expectation = self.expectation(description: "Error added to log")
        let errorMessage = "Test error"
        let errorDetails = "A sensitive error occurred"

        consolidatedLog.addError(message: errorMessage, error: errorDetails) {
            expectation.fulfill()
        }

        waitForExpectations(timeout: 1)
        XCTAssertTrue(
            consolidatedLog.string.contains(errorMessage),
            "Log should include the error message."
        )
    }

    func testStringOutput() {
        let expectation = self.expectation(description: "Log files added")
        let mockFile = createMockFile(content: content, fileName: "\(generateRandomName()).txt")
        consolidatedLog.addLogFiles(fileURLs: [mockFile]) {
            expectation.fulfill()
            let output = self.consolidatedLog.string
            XCTAssertFalse(output.isEmpty, "Output string should include redacted log content.")
        }
        waitForExpectations(timeout: 1)
    }

    // MARK: - Private functions

    private func createMockFile(content: String, fileName: String) -> URL {
        let tempDirectory = FileManager.default.temporaryDirectory
        let fileURL = tempDirectory.appendingPathComponent(fileName)

        do {
            try content.write(to: fileURL, atomically: true, encoding: .utf8)
            createdMockFiles.append(fileURL)
        } catch {
            XCTFail("Failed to create mock file: \(error)")
        }
        return fileURL
    }

    private func generateRandomName() -> String {
        let characterSet = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
        let randomName = (0 ..< 6).compactMap { _ in characterSet.randomElement() }
        return String(randomName)
    }
}

extension ConsolidatedApplicationLogTests {
    private var content: String {
        return """
        MullvadVPN version xxxx.x
        [22/11/2024 @ 08:52:22][AppDelegate][debug] Registered app refresh task.
        [22/11/2024 @ 08:52:22][AppDelegate][debug] Registered address cache update task.
        [22/11/2024 @ 08:52:22][AppDelegate][debug] Registered private key rotation task.
        [22/11/2024 @ 08:52:23][TunnelManager][debug] Refresh device state
        and tunnel status
        due to application becoming active.
        [22/11/2024 @ 08:52:23][RelayCacheTracker][debug] Start periodic relay updates.
        [22/11/2024 @ 08:52:23][AddressCache.Tracker][debug] Start periodic address cache updates.
        [22/11/2024 @ 08:52:23][AddressCache.Tracker][debug] Schedule address cache update at 23/11/2024 @ 08:49:52.
        [22/11/2024 @ 08:52:23][AppDelegate][debug] Attempted migration from UI Process, but found nothing to do.
        [22/11/2024 @ 08:52:23][TunnelManager][debug] Refresh tunnel status for new tunnel.
        [22/11/2024 @ 08:52:23][REST.NetworkOperation][debug] name=get-access-token.2
        Send request
        to /auth/v1/token via 127.0.0.1 using encrypted-dns-url-session.
        [22/11/2024 @ 08:52:23][ApplicationRouter][debug] Presenting .main.
        [22/11/2024 @ 08:52:23][REST.NetworkOperation][debug] name=get-access-token.2 Response: 200.
        [22/11/2024 @ 08:52:23][AppDelegate][debug] Finished initialization.
        """
    }
}
