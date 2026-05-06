//
//  RustLoggingTests.swift
//  MullvadRustRuntimeTests
//
//  Created by Emīls on 2026-04-29.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadLogging
import XCTest

@testable import MullvadRustRuntime

/// Captured log entry from the Rust logging callback.
private struct CapturedLog {
    let level: UInt8
    let target: String
    let message: String
}

/// Mutable storage shared between the test and the C callback via an opaque pointer.
private final class LogCapture {
    var logs: [CapturedLog] = []
}

/// C-compatible callback that captures log data into the `LogCapture` context.
private func testLogCallback(
    context: UnsafeMutableRawPointer?,
    level: UInt8,
    targetPtr: UnsafePointer<CChar>?,
    messagePtr: UnsafePointer<CChar>?
) {
    guard let context, let targetPtr, let messagePtr else { return }
    let capture = Unmanaged<LogCapture>.fromOpaque(context).takeUnretainedValue()
    capture.logs.append(
        CapturedLog(
            level: level,
            target: String(cString: targetPtr),
            message: String(cString: messagePtr)
        ))
}

/// Tests for the Rust → Swift logging callback mechanism.
final class RustLoggingTests: XCTestCase {
    /// Verify that a context pointer round-trips correctly through the C callback,
    /// and that log parameters are passed through faithfully.
    func testCallbackContextRoundTrip() {
        let capture = LogCapture()
        let context = Unmanaged.passUnretained(capture).toOpaque()

        // Simulate what Rust does: call the callback directly with test data
        "mullvad_ios::logging".withCString { target in
            "Test message from Rust".withCString { message in
                testLogCallback(context: context, level: 3, targetPtr: target, messagePtr: message)
            }
        }

        XCTAssertEqual(capture.logs.count, 1, "Expected exactly one log entry")

        let entry = capture.logs[0]
        XCTAssertEqual(entry.level, 3)
        XCTAssertEqual(entry.target, "mullvad_ios::logging")
        XCTAssertEqual(entry.message, "Test message from Rust")
    }

    /// Verify that null parameters are handled gracefully (callback returns early).
    func testCallbackIgnoresNullParameters() {
        let capture = LogCapture()
        let context = Unmanaged.passUnretained(capture).toOpaque()

        // Null target and message — callback should bail out
        testLogCallback(context: context, level: 1, targetPtr: nil, messagePtr: nil)
        XCTAssertTrue(capture.logs.isEmpty, "Callback should ignore null parameters")

        // Null context — callback should bail out
        "target".withCString { target in
            "message".withCString { message in
                testLogCallback(context: nil, level: 1, targetPtr: target, messagePtr: message)
            }
        }
        XCTAssertTrue(capture.logs.isEmpty, "Callback should ignore null context")
    }

}
