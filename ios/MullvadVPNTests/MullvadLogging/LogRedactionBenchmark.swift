//
//  LogRedactionBenchmark.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-04-27.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import XCTest

@testable import MullvadRustRuntime

class LogRedactionBenchmarkTests: XCTestCase {

    // Short messages with IPv4
    let shortIPv4Message = "Connected to 192.168.1.1 successfully"
    let shortIPv4FullLine = "[2026-01-29 10:30:45][TunnelManager][info] Connected to 192.168.1.1 successfully"

    // Short messages with IPv6
    let shortIPv6Message = "Connected to 2001:db8:85a3::8a2e:370:7334 successfully"
    let shortIPv6FullLine =
        "[2026-01-29 10:30:45][TunnelManager][info] Connected to 2001:db8:85a3::8a2e:370:7334 successfully"

    // Long messages with multiple IPs
    let longMessage = """
        Tunnel connection established. Primary endpoint: 192.168.1.1:51820, \
        backup endpoint: 10.0.0.1:51820. IPv6 addresses: 2001:db8:85a3::8a2e:370:7334, \
        fe80::1%en0. Account verification completed for user session. \
        DNS servers configured: 192.168.1.53, 8.8.8.8, 2001:4860:4860::8888. \
        Gateway: 192.168.1.254. Network interface ready.
        """

    let longFullLine = """
        [2026-01-29 10:30:45][TunnelManager][info] pid=12345 session=abc123 \
        Tunnel connection established. Primary endpoint: 192.168.1.1:51820, \
        backup endpoint: 10.0.0.1:51820. IPv6 addresses: 2001:db8:85a3::8a2e:370:7334, \
        fe80::1%en0. Account verification completed for user session. \
        DNS servers configured: 192.168.1.53, 8.8.8.8, 2001:4860:4860::8888. \
        Gateway: 192.168.1.254. Network interface ready.
        """

    // Messages with no matches (common case)
    let noMatchMessage = "Application started successfully"
    let noMatchFullLine = "[2026-01-29 10:30:45][AppDelegate][debug] Application started successfully"

    // Account number messages
    let accountMessage = "Login attempt for account 1234567890123456"
    let accountFullLine = "[2026-01-29 10:30:45][Auth][info] Login attempt for account 1234567890123456"

    lazy var logs: [String] = [
        self.shortIPv4Message,
//        self.shortIPv6Message,
//        self.shortIPv6FullLine,
//        self.longMessage,
        self.noMatchMessage,
        self.noMatchFullLine,
    ]

    func testPerformanceLogRedaction() {
        let consolidatedApplicationLog = ConsolidatedApplicationLog(
            redactCustomStrings: nil,
            redactContainerPathsForSecurityGroupIdentifiers: [UUID().uuidString],
            bufferSize: ApplicationConfiguration.logMaximumFileSize
        )

        measure {
            for _ in 0...1000 {
                for entry in logs {
                    _ = consolidatedApplicationLog.redact(string: entry)
                }
            }

        }
    }

    func testNewRedactor() {
        let redaction = LogRedaction()
        let consolidatedApplicationLog = ConsolidatedApplicationLog(
            redactCustomStrings: nil,
            redactContainerPathsForSecurityGroupIdentifiers: [UUID().uuidString],
            bufferSize: ApplicationConfiguration.logMaximumFileSize
        )

        for entry in logs {
            XCTAssertEqual(redaction.redact(string: entry), consolidatedApplicationLog.redact(string: entry))
        }
    }
}
