//
//  LogRedactorBenchmarkTests.swift
//  MullvadVPNTests
//
//  Created by Emīls on 2026-01-29.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadLogging
import MullvadRustRuntime
import XCTest

/// Benchmark comparing Swift and Rust log redaction over a representative mix of log entries.
///
/// Each benchmark run redacts all test inputs, giving a realistic picture of per-log-batch cost.
final class LogRedactorBenchmarkTests: XCTestCase {
    let swiftRedactor = LogRedactor.shared
    let rustRedactor = RustLogRedactor()

    // MARK: - Test Data

    /// Representative mix of log entries covering all redaction patterns and the common no-match case.
    let logEntries = [
        // IPv4
        "[2026-01-29 10:30:45][TunnelManager][info] Connected to 192.168.1.1 successfully",
        // IPv6
        "[2026-01-29 10:30:45][TunnelManager][info] Connected to 2001:db8:85a3::8a2e:370:7334 successfully",
        // Account number
        "[2026-01-29 10:30:45][Auth][info] Login attempt for account 1234567890123456",
        // No match (common case)
        "[2026-01-29 10:30:45][AppDelegate][debug] Application started successfully",
        // Long line with multiple IPs
        """
        [2026-01-29 10:30:45][TunnelManager][info] pid=12345 session=abc123 \
        Tunnel connection established. Primary endpoint: 192.168.1.1:51820, \
        backup endpoint: 10.0.0.1:51820. IPv6 addresses: 2001:db8:85a3::8a2e:370:7334, \
        fe80::1%en0. Account verification completed for user session. \
        DNS servers configured: 192.168.1.53, 8.8.8.8, 2001:4860:4860::8888. \
        Gateway: 192.168.1.254. Network interface ready.
        """,
    ]

    // MARK: - Benchmarks

    func testBenchmarkSwiftRedactor() {
        measure {
            for _ in 0..<10_000 {
                for entry in logEntries {
                    _ = swiftRedactor.redact(entry)
                }
            }
        }
    }

    func testBenchmarkRustRedactor() {
        measure {
            for _ in 0..<10_000 {
                for entry in logEntries {
                    _ = rustRedactor.redact(entry)
                }
            }
        }
    }
}
