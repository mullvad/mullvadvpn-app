//
//  ConsolidatedApplicationLogTests.swift
//  MullvadVPNTests
//
//  Created by Mojgan on 2024-11-21.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadLogging
import MullvadRustRuntime
import XCTest

final class ConsolidatedApplicationLogTests: XCTestCase, @unchecked Sendable {
    nonisolated(unsafe) var consolidatedLog: ConsolidatedApplicationLog!
    var createdMockFiles: [URL] = []
    let kRedactedPlaceholder = "[REDACTED]"

    override func setUp() {
        super.setUp()
        consolidatedLog = ConsolidatedApplicationLog(
            redactor: RustLogRedactor(),
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

    func testAddLogFiles() async {
        var string = ""
        let expectation = self.expectation(description: "Log files added")
        let mockFile = createMockFile(content: content, fileName: "\(generateRandomName()).txt")

        consolidatedLog.addLogFiles(fileURLs: [mockFile]) {
            expectation.fulfill()
        }

        await fulfillment(of: [expectation], timeout: 1)
        consolidatedLog.write(to: &string)
        XCTAssertTrue(
            consolidatedLog.string.contains(string),
            "Log should contain the file content."
        )
    }

    func testAddError() async {
        let expectation = self.expectation(description: "Error added to log")
        let errorMessage = "Test error"
        let errorDetails = "A sensitive error occurred"

        consolidatedLog.addError(message: errorMessage, error: errorDetails) {
            expectation.fulfill()
        }

        await fulfillment(of: [expectation], timeout: 1)
        XCTAssertTrue(
            consolidatedLog.string.contains(errorMessage),
            "Log should include the error message."
        )
    }

    func testStringOutput() async {
        let expectation = self.expectation(description: "Log files added")
        let mockFile = createMockFile(content: content, fileName: "\(generateRandomName()).txt")
        consolidatedLog.addLogFiles(fileURLs: [mockFile]) {
            expectation.fulfill()
            let output = self.consolidatedLog.string
            XCTAssertFalse(output.isEmpty, "Output string should include redacted log content.")
        }
        await fulfillment(of: [expectation], timeout: 1)
    }

    /// Verifies that log files from pre-2026.3 releases (which lack on-the-fly redaction)
    /// have their sensitive content fully redacted at collection time.
    func testOldLogFileContentIsRedactedAtCollectionTime() async {
        let expectation = self.expectation(description: "Old log file processed")
        let mockFile = createMockFile(content: oldReleaseLogContent, fileName: "\(generateRandomName()).log")

        consolidatedLog.addLogFiles(fileURLs: [mockFile]) {
            expectation.fulfill()
        }

        await fulfillment(of: [expectation], timeout: 1)
        let output = consolidatedLog.string

        // IPs that were NOT redacted on-the-fly in the old release must be redacted now
        XCTAssertFalse(output.contains("192.168.1.1"), "IPv4 address should be redacted")
        XCTAssertFalse(output.contains("10.64.0.1"), "IPv4 address should be redacted")
        XCTAssertFalse(output.contains("2001:db8:85a3::8a2e:370:7334"), "IPv6 address should be redacted")
        XCTAssertFalse(output.contains("1234567890123456"), "Account number should be redacted")

        // Redaction placeholders should be present
        XCTAssertTrue(output.contains("[REDACTED]"), "Should contain IP redaction placeholders")
        XCTAssertTrue(output.contains("[REDACTED ACCOUNT NUMBER]"), "Should contain account redaction placeholder")

        // Non-sensitive content should survive
        XCTAssertTrue(output.contains("MullvadVPN version 2024.5"), "Version header should be preserved")
        XCTAssertTrue(output.contains("Refresh device state"), "Normal log text should be preserved")
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
        let randomName = (0..<6).compactMap { _ in characterSet.randomElement() }
        return String(randomName)
    }
}

extension ConsolidatedApplicationLogTests {
    private var content: String {
        """
        MullvadVPN version xxxx.x
        [22/11/2024 @ 08:52:22][AppDelegate][debug] Registered app refresh task.
        [22/11/2024 @ 08:52:22][AppDelegate][debug] Registered address cache update task.
        [22/11/2024 @ 08:52:23][TunnelManager][debug] Refresh device state
        and tunnel status
        due to application becoming active.
        [22/11/2024 @ 08:52:23][AppDelegate][debug] Finished initialization.
        """
    }

    /// Simulates a log file from a pre-2026.3 release that did NOT have on-the-fly redaction.
    /// Contains raw IPs, account numbers, and other sensitive data that must be redacted
    /// at collection time.
    private var oldReleaseLogContent: String {
        """
        MullvadVPN version 2024.5
        [15/03/2025 @ 10:30:01][AppDelegate][debug] Registered app refresh task.
        [15/03/2025 @ 10:30:01][TunnelManager][debug] Refresh device state and tunnel status.
        [15/03/2025 @ 10:30:02][REST.NetworkOperation][debug] name=get-access-token.2 \
        Send request to /auth/v1/token via 192.168.1.1 using encrypted-dns-url-session.
        [15/03/2025 @ 10:30:02][TunnelManager][info] Connected to 2001:db8:85a3::8a2e:370:7334 successfully.
        [15/03/2025 @ 10:30:02][TunnelManager][info] Tunnel established. Local IP: 10.64.0.1
        [15/03/2025 @ 10:30:03][Auth][info] Login attempt for account 1234567890123456 completed.
        [15/03/2025 @ 10:30:03][AppDelegate][debug] Finished initialization.
        """
    }
}
