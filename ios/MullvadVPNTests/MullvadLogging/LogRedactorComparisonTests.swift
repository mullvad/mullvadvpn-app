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
    let rustRedactor = RustLogRedactor()

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
            let rustResult = rustRedactor.redact(input)
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
            let rustResult = rustRedactor.redact(input)
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
            let rustResult = rustRedactor.redact(input)
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
            let rustResult = rustRedactor.redact(input)
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
            let rustResult = rustRedactor.redact(input)
            XCTAssertEqual(
                swiftResult,
                rustResult,
                "Mismatch for input: \(input)"
            )
        }
    }

    // MARK: - Container Path Tests

    func testContainerPathRedaction() {
        let containerPath = "/var/mobile/Containers/Shared/AppGroup/ABCD1234-5678-90AB-CDEF-1234567890AB"
        let redactor = RustLogRedactor(containerPaths: [containerPath])
        let input = "Reading file at \(containerPath)/Logs/app.log"
        let result = redactor.redact(input)
        XCTAssertFalse(result.contains(containerPath))
        XCTAssertTrue(result.contains("[REDACTED CONTAINER PATH]"))
    }

    func testMultipleContainerPaths() {
        let path1 = "/var/mobile/Containers/Shared/AppGroup/AAAA1111-2222-3333-4444-555566667777"
        let path2 = "/var/mobile/Containers/Data/Application/BBBB1111-2222-3333-4444-555566667777"
        let redactor = RustLogRedactor(containerPaths: [path1, path2])
        let input = "Read \(path1)/a.log and \(path2)/b.log"
        let result = redactor.redact(input)
        XCTAssertFalse(result.contains(path1))
        XCTAssertFalse(result.contains(path2))
        XCTAssertEqual(
            result,
            "Read [REDACTED CONTAINER PATH]/a.log and [REDACTED CONTAINER PATH]/b.log"
        )
    }

    func testEmptyInput() {
        let result = rustRedactor.redact("")
        XCTAssertEqual(result, "")
    }

    func testNoMatchInputUnchanged() {
        let input = "Application started successfully"
        let result = rustRedactor.redact(input)
        XCTAssertEqual(result, input)
    }

    // MARK: - MAC Address Tests

    func testMACAddressRedactionColonSeparated() {
        XCTAssertEqual(
            rustRedactor.redact("Interface MAC: aa:bb:cc:dd:ee:ff"),
            "Interface MAC: [REDACTED]"
        )
    }

    func testMACAddressRedactionDashSeparated() {
        XCTAssertEqual(
            rustRedactor.redact("Interface MAC: AA-BB-CC-DD-EE-FF"),
            "Interface MAC: [REDACTED]"
        )
    }

    func testMACAddressRedactionMixedCase() {
        XCTAssertEqual(
            rustRedactor.redact("MAC is 0A:1b:2C:3d:4E:5f on en0"),
            "MAC is [REDACTED] on en0"
        )
    }

    func testRedactorWithNoContainerPaths() {
        let redactor = RustLogRedactor()
        let input = "Connected to 192.168.1.1 for account 1234567890123456"
        let result = redactor.redact(input)
        XCTAssertTrue(result.contains("[REDACTED]"))
        XCTAssertTrue(result.contains("[REDACTED ACCOUNT NUMBER]"))
    }

    func testCombinedIPsAccountsAndContainerPaths() {
        let containerPath = "/var/mobile/Containers/Shared/AppGroup/ABCD1234-5678-90AB-CDEF-1234567890AB"
        let redactor = RustLogRedactor(containerPaths: [containerPath])
        let input = "User 1234567890123456 at 10.0.0.1 wrote to \(containerPath)/data.db via 2001:db8::1"
        let result = redactor.redact(input)
        XCTAssertFalse(result.contains("1234567890123456"))
        XCTAssertFalse(result.contains("10.0.0.1"))
        XCTAssertFalse(result.contains(containerPath))
        XCTAssertFalse(result.contains("2001:db8::1"))
    }
}
