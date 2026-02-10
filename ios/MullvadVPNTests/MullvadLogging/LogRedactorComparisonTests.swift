//
//  LogRedactorComparisonTests.swift
//  MullvadVPNTests
//
//  Created by Claude on 2026-01-30.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadLogging
import MullvadRustRuntime
import XCTest

/// Tests that verify Rust FFI redaction produces identical output to Swift's LogRedactor.
final class LogRedactorComparisonTests: XCTestCase {
    // MARK: - IPv4 Tests

    func testIPv4RedactionMatches() {
        let inputs = [
            "Connected to 192.168.1.1 successfully",
            "Primary: 192.168.1.1, Secondary: 10.0.0.1",
            "Endpoint: 192.168.1.1:51820",
            "Range: 0.0.0.0 to 255.255.255.255",
            "Invalid IP: 999.999.999.999",  // Should not match
        ]

        for input in inputs {
            let swiftResult = LogRedactor.shared.redact(input)
            let rustResult = RustLogRedactor.redact(input)
            XCTAssertEqual(
                swiftResult,
                rustResult,
                "Mismatch for input: \(input)"
            )
        }
    }

    // MARK: - IPv6 Tests

    func testIPv6RedactionMatches() {
        let inputs = [
            "Address: 2001:0db8:85a3:0000:0000:8a2e:0370:7334",
            "Address: 2001:db8:85a3::8a2e:370:7334",
            "Loopback: ::1",
            "Link-local: fe80::1%en0",
            "Mapped: ::ffff:192.168.1.1",
            "Full: 1:2:3:4:5:6:7:8",
            "Compressed: 1::8",
            "Empty: ::",
        ]

        for input in inputs {
            let swiftResult = LogRedactor.shared.redact(input)
            let rustResult = RustLogRedactor.redact(input)
            XCTAssertEqual(
                swiftResult,
                rustResult,
                "Mismatch for input: \(input)"
            )
        }
    }

    // MARK: - Account Number Tests

    func testAccountRedactionMatches() {
        let inputs = [
            "Account: 1234567890123456",
            "Short number: 123456789012345",  // 15 digits - should not match
            "Long number: 12345678901234567890",  // 20 digits
        ]

        for input in inputs {
            let swiftResult = LogRedactor.shared.redact(input)
            let rustResult = RustLogRedactor.redact(input)
            XCTAssertEqual(
                swiftResult,
                rustResult,
                "Mismatch for input: \(input)"
            )
        }
    }

    // MARK: - Combined Tests

    func testCombinedRedactionMatches() {
        let inputs = [
            "User 1234567890123456 connected to 192.168.1.1 via 2001:db8::1",
            "Application started successfully",
            "",
            """
            Tunnel connection established. Primary endpoint: 192.168.1.1:51820, \
            backup endpoint: 10.0.0.1:51820. IPv6 addresses: 2001:db8:85a3::8a2e:370:7334, \
            fe80::1%en0. Account verification completed for user session. \
            DNS servers configured: 192.168.1.53, 8.8.8.8, 2001:4860:4860::8888. \
            Gateway: 192.168.1.254. Network interface ready.
            """,
        ]

        for input in inputs {
            let swiftResult = LogRedactor.shared.redact(input)
            let rustResult = RustLogRedactor.redact(input)
            XCTAssertEqual(
                swiftResult,
                rustResult,
                "Mismatch for input: \(input)"
            )
        }
    }

    // MARK: - Log Line Format Tests

    func testFullLogLineRedactionMatches() {
        let inputs = [
            "[2026-01-29 10:30:45][TunnelManager][info] Connected to 192.168.1.1 successfully",
            "[2026-01-29 10:30:45][TunnelManager][info] Connected to 2001:db8:85a3::8a2e:370:7334 successfully",
            "[2026-01-29 10:30:45][Auth][info] Login attempt for account 1234567890123456",
            "[2026-01-29 10:30:45][AppDelegate][debug] Application started successfully",
        ]

        for input in inputs {
            let swiftResult = LogRedactor.shared.redact(input)
            let rustResult = RustLogRedactor.redact(input)
            XCTAssertEqual(
                swiftResult,
                rustResult,
                "Mismatch for input: \(input)"
            )
        }
    }
}
