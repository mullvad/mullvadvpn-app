//
//  LogRedactorComparisonTests.swift
//  MullvadVPNTests
//
//  Created by Claude on 2026-01-30.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadRustRuntime
import XCTest

/// Tests verifying the Rust FFI log redactor produces correct output.
final class LogRedactorComparisonTests: XCTestCase {
    let rustRedactor = RustLogRedactor()

    // MARK: - IPv4 Tests

    func testIPv4Redaction() {
        XCTAssertEqual(
            rustRedactor.redact("Connected to 192.168.1.1 successfully"),
            "Connected to [REDACTED] successfully"
        )
        XCTAssertEqual(
            rustRedactor.redact("Primary: 192.168.1.1, Secondary: 10.0.0.1"),
            "Primary: [REDACTED], Secondary: [REDACTED]"
        )
        XCTAssertEqual(
            rustRedactor.redact("Endpoint: 192.168.1.1:51820"),
            "Endpoint: [REDACTED]:51820"
        )
        XCTAssertEqual(
            rustRedactor.redact("Range: 0.0.0.0 to 255.255.255.255"),
            "Range: [REDACTED] to [REDACTED]"
        )
        // Should not match invalid IPs
        XCTAssertEqual(
            rustRedactor.redact("Invalid IP: 999.999.999.999"),
            "Invalid IP: 999.999.999.999"
        )
    }

    // MARK: - IPv6 Tests

    func testIPv6Redaction() {
        let cases: [(input: String, shouldRedact: Bool)] = [
            ("Address: 2001:0db8:85a3:0000:0000:8a2e:0370:7334", true),
            ("Address: 2001:db8:85a3::8a2e:370:7334", true),
            ("Loopback: ::1", true),
            ("Link-local: fe80::1%en0", true),
            ("Mapped: ::ffff:192.168.1.1", true),
            ("Full: 1:2:3:4:5:6:7:8", true),
            ("Compressed: 1::8", true),
            ("Empty: ::", true),
        ]

        for (input, shouldRedact) in cases {
            let result = rustRedactor.redact(input)
            if shouldRedact {
                XCTAssertTrue(
                    result.contains("[REDACTED]"),
                    "Expected redaction for: \(input), got: \(result)"
                )
            }
        }
    }

    // MARK: - Account Number Tests

    func testAccountRedaction() {
        XCTAssertEqual(
            rustRedactor.redact("Account: 1234567890123456"),
            "Account: [REDACTED ACCOUNT NUMBER]"
        )
        // 15 digits — should NOT match
        XCTAssertEqual(
            rustRedactor.redact("Short number: 123456789012345"),
            "Short number: 123456789012345"
        )
    }

    // MARK: - Combined Tests

    func testCombinedRedaction() {
        let input = "User 1234567890123456 connected to 192.168.1.1 via 2001:db8::1"
        let result = rustRedactor.redact(input)
        XCTAssertFalse(result.contains("1234567890123456"))
        XCTAssertFalse(result.contains("192.168.1.1"))
        XCTAssertFalse(result.contains("2001:db8::1"))
    }

    func testFullLogLineRedaction() {
        let inputs = [
            "[2026-01-29 10:30:45][TunnelManager][info] Connected to 192.168.1.1 successfully",
            "[2026-01-29 10:30:45][TunnelManager][info] Connected to 2001:db8:85a3::8a2e:370:7334 successfully",
            "[2026-01-29 10:30:45][Auth][info] Login attempt for account 1234567890123456",
        ]

        for input in inputs {
            let result = rustRedactor.redact(input)
            XCTAssertTrue(result.contains("[REDACTED"), "Expected redaction in: \(input)")
            // Metadata should be preserved
            XCTAssertTrue(result.contains("[2026-01-29 10:30:45]"))
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

    // MARK: - Edge Cases

    func testEmptyInput() {
        XCTAssertEqual(rustRedactor.redact(""), "")
    }

    func testNoMatchInputUnchanged() {
        let input = "Application started successfully"
        XCTAssertEqual(rustRedactor.redact(input), input)
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
