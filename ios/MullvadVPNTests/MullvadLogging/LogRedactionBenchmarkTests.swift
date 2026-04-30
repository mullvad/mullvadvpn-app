//
//  LogRedactionBenchmarkTests.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-04-27.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import XCTest

@testable import MullvadLogging
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

    // Container path
    let containerPath = {
        [UUID().uuidString]
            .compactMap { securityGroupIdentifier -> URL? in
                FileManager.default
                    .containerURL(forSecurityApplicationGroupIdentifier: securityGroupIdentifier)
            }
    }

    lazy var logs: [String] = [
        self.shortIPv4Message,
        self.shortIPv6Message,
        self.shortIPv6FullLine,
        self.longMessage,
        self.noMatchMessage,
        self.noMatchFullLine,
        ["\(String(describing: containerPath))", "net.mullvad.MullvadVPN_29-04-2026T15:00:01.log"].joined(),
    ]

    func testPerformanceSwiftLogRedaction() {
        let options = XCTMeasureOptions()
        options.iterationCount = 10000

        let redactor = SwiftLogRedactor()
        let containerPaths = [UUID().uuidString]
            .compactMap { securityGroupIdentifier -> URL? in
                FileManager.default
                    .containerURL(forSecurityApplicationGroupIdentifier: securityGroupIdentifier)
            }

        measure(options: options) {
            for entry in logs {
                _ = redactor.redact(entry, using: [.accountNumbers, .ipv4, .ipv6, .containerPaths(containerPaths)])
            }
        }
    }

    func testPerformanceRustLogRedaction() {
        let options = XCTMeasureOptions()
        options.iterationCount = 10000

        let redactor = RustLogRedactor()
        let containerPaths = [UUID().uuidString]
            .compactMap { securityGroupIdentifier -> URL? in
                FileManager.default
                    .containerURL(forSecurityApplicationGroupIdentifier: securityGroupIdentifier)
            }

        measure(options: options) {
            for entry in logs {
                _ = redactor.redact(
                    entry,
                    using: [
                        .accountNumbers,
                        .ipv4,
                        .ipv6,
                        .containerPaths(containerPaths),
                    ])
            }
        }
    }

    func testRedactionMigrationNoBehaviorRegression() {
        let rustLogRedactor = RustLogRedactor()
        let swiftLogRedactor = SwiftLogRedactor()
        let rules: [RedactionRules] = [.accountNumbers, .ipv4, .ipv6, .containerPaths(containerPath())]

        for entry in logs {
            XCTAssertEqual(
                rustLogRedactor.redact(entry, using: rules),
                swiftLogRedactor.redact(entry, using: rules))
        }
    }
}
