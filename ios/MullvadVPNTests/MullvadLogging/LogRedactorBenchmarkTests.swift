//
//  LogRedactorBenchmarkTests.swift
//  MullvadVPNTests
//
//  Created by Emīls on 2026-01-29.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadLogging
import XCTest

/// Benchmark tests for LogRedactor to compare regex performance on:
/// - Message-only strings (what we redact with structured logging)
/// - Full log lines (what we would redact without structured logging)
///
/// Run these tests to measure the performance benefit of redacting only the message portion.
final class LogRedactorBenchmarkTests: XCTestCase {
    let redactor = LogRedactor.shared

    // MARK: - Test Data

    // Short messages with IPv4
    let shortIPv4Message = "Connected to 192.168.1.1 successfully"
    let shortIPv4FullLine = "[2026-01-29 10:30:45][TunnelManager][info] Connected to 192.168.1.1 successfully"

    // Short messages with IPv6
    let shortIPv6Message = "Connected to 2001:db8:85a3::8a2e:370:7334 successfully"
    let shortIPv6FullLine = "[2026-01-29 10:30:45][TunnelManager][info] Connected to 2001:db8:85a3::8a2e:370:7334 successfully"

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

    // MARK: - IPv4 Benchmarks

    func testBenchmarkIPv4MessageOnly() {
        measure {
            for _ in 0..<10000 {
                _ = redactor.redact(shortIPv4Message)
            }
        }
    }

    func testBenchmarkIPv4FullLine() {
        measure {
            for _ in 0..<10000 {
                _ = redactor.redact(shortIPv4FullLine)
            }
        }
    }

    // MARK: - IPv6 Benchmarks (known to be slowest)

    func testBenchmarkIPv6MessageOnly() {
        measure {
            for _ in 0..<10000 {
                _ = redactor.redact(shortIPv6Message)
            }
        }
    }

    func testBenchmarkIPv6FullLine() {
        measure {
            for _ in 0..<10000 {
                _ = redactor.redact(shortIPv6FullLine)
            }
        }
    }

    // IPv6-only redaction to isolate the regex cost
    func testBenchmarkIPv6RegexOnlyMessageOnly() {
        measure {
            for _ in 0..<10000 {
                _ = redactor.redactIPv6(shortIPv6Message)
            }
        }
    }

    func testBenchmarkIPv6RegexOnlyFullLine() {
        measure {
            for _ in 0..<10000 {
                _ = redactor.redactIPv6(shortIPv6FullLine)
            }
        }
    }

    // MARK: - Long Message Benchmarks

    func testBenchmarkLongMessageOnly() {
        measure {
            for _ in 0..<10000 {
                _ = redactor.redact(longMessage)
            }
        }
    }

    func testBenchmarkLongFullLine() {
        measure {
            for _ in 0..<10000 {
                _ = redactor.redact(longFullLine)
            }
        }
    }

    // MARK: - No Match Benchmarks (common case - measures overhead)

    func testBenchmarkNoMatchesMessageOnly() {
        measure {
            for _ in 0..<10000 {
                _ = redactor.redact(noMatchMessage)
            }
        }
    }

    func testBenchmarkNoMatchesFullLine() {
        measure {
            for _ in 0..<10000 {
                _ = redactor.redact(noMatchFullLine)
            }
        }
    }

    // MARK: - Account Number Benchmarks

    func testBenchmarkAccountMessageOnly() {
        measure {
            for _ in 0..<10000 {
                _ = redactor.redact(accountMessage)
            }
        }
    }

    func testBenchmarkAccountFullLine() {
        measure {
            for _ in 0..<10000 {
                _ = redactor.redact(accountFullLine)
            }
        }
    }
}
