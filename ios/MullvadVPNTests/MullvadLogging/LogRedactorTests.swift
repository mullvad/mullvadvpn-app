//
//  LogRedactorTests.swift
//  MullvadVPNTests
//
//  Created by Emīls on 2026-01-29.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadLogging
import XCTest

final class LogRedactorTests: XCTestCase {
    let redactor = LogRedactor.shared

    // MARK: - IPv4 Tests

    func testRedactsIPv4Address() {
        let input = "Connected to 192.168.1.1 successfully"
        let expected = "Connected to [REDACTED] successfully"
        XCTAssertEqual(redactor.redact(input), expected)
    }

    func testRedactsMultipleIPv4Addresses() {
        let input = "Primary: 192.168.1.1, Secondary: 10.0.0.1"
        let expected = "Primary: [REDACTED], Secondary: [REDACTED]"
        XCTAssertEqual(redactor.redact(input), expected)
    }

    func testRedactsIPv4WithPort() {
        let input = "Endpoint: 192.168.1.1:51820"
        let expected = "Endpoint: [REDACTED]:51820"
        XCTAssertEqual(redactor.redact(input), expected)
    }

    func testDoesNotRedactInvalidIPv4() {
        let input = "Invalid IP: 999.999.999.999"
        // 999 is not a valid octet, so it should not match
        XCTAssertEqual(redactor.redact(input), input)
    }

    func testRedactsBoundaryIPv4Values() {
        let input = "Range: 0.0.0.0 to 255.255.255.255"
        let expected = "Range: [REDACTED] to [REDACTED]"
        XCTAssertEqual(redactor.redact(input), expected)
    }

    // MARK: - IPv6 Tests

    func testRedactsFullIPv6Address() {
        let input = "Address: 2001:0db8:85a3:0000:0000:8a2e:0370:7334"
        let expected = "Address: [REDACTED]"
        XCTAssertEqual(redactor.redact(input), expected)
    }

    func testRedactsCompressedIPv6Address() {
        let input = "Address: 2001:db8:85a3::8a2e:370:7334"
        let expected = "Address: [REDACTED]"
        XCTAssertEqual(redactor.redact(input), expected)
    }

    func testRedactsIPv6LoopbackAddress() {
        let input = "Loopback: ::1"
        let expected = "Loopback: [REDACTED]"
        XCTAssertEqual(redactor.redact(input), expected)
    }

    func testRedactsIPv6LinkLocalWithZoneIndex() {
        let input = "Link-local: fe80::1%en0"
        let expected = "Link-local: [REDACTED]"
        XCTAssertEqual(redactor.redact(input), expected)
    }

    func testRedactsIPv4MappedIPv6Address() {
        let input = "Mapped: ::ffff:192.168.1.1"
        let expected = "Mapped: [REDACTED]"
        XCTAssertEqual(redactor.redact(input), expected)
    }

    // MARK: - Account Number Tests

    func testRedactsAccountNumber() {
        let input = "Account: 1234567890123456"
        let expected = "Account: [REDACTED ACCOUNT NUMBER]"
        XCTAssertEqual(redactor.redact(input), expected)
    }

    func testDoesNotRedactShorterNumberSequence() {
        let input = "Short number: 123456789012345"  // 15 digits
        XCTAssertEqual(redactor.redact(input), input)
    }

    func testRedactsOnlyFirst16DigitsOfLongerSequence() {
        let input = "Long number: 12345678901234567890"  // 20 digits
        let expected = "Long number: [REDACTED ACCOUNT NUMBER]7890"
        XCTAssertEqual(redactor.redact(input), expected)
    }

    // MARK: - Combined Tests

    func testRedactsMultipleTypesOfSensitiveData() {
        let input = "User 1234567890123456 connected to 192.168.1.1 via 2001:db8::1"
        let expected = "User [REDACTED ACCOUNT NUMBER] connected to [REDACTED] via [REDACTED]"
        XCTAssertEqual(redactor.redact(input), expected)
    }

    func testPreservesNonSensitiveContent() {
        let input = "Application started successfully at 10:30:45"
        XCTAssertEqual(redactor.redact(input), input)
    }

    func testHandlesEmptyString() {
        XCTAssertEqual(redactor.redact(""), "")
    }

    // MARK: - Individual Redaction Method Tests

    func testRedactIPv4Only() {
        let input = "IP: 192.168.1.1, Account: 1234567890123456"
        let expected = "IP: [REDACTED], Account: 1234567890123456"
        XCTAssertEqual(redactor.redactIPv4(input), expected)
    }

    func testRedactIPv6Only() {
        let input = "IPv4: 192.168.1.1, IPv6: 2001:db8::1"
        let expected = "IPv4: 192.168.1.1, IPv6: [REDACTED]"
        XCTAssertEqual(redactor.redactIPv6(input), expected)
    }

    func testRedactAccountNumberOnly() {
        let input = "IP: 192.168.1.1, Account: 1234567890123456"
        let expected = "IP: 192.168.1.1, Account: [REDACTED ACCOUNT NUMBER]"
        XCTAssertEqual(redactor.redactAccountNumber(input), expected)
    }
}
