//
//  RustLogRedactor.swift
//  MullvadRustRuntime
//
//  Created by Claude on 2026-01-30.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Swift wrapper for Rust log redaction FFI.
///
/// This provides the same redaction functionality as `LogRedactor` but implemented in Rust,
/// allowing performance comparison between Swift and Rust regex implementations.
public enum RustLogRedactor {
    /// Redact sensitive information from a string using Rust regex.
    ///
    /// Redacts:
    /// - IPv4 addresses → `[REDACTED]`
    /// - IPv6 addresses → `[REDACTED]`
    /// - Account numbers (16-digit sequences) → `[REDACTED ACCOUNT NUMBER]`
    ///
    /// - Parameter string: The string to redact.
    /// - Returns: A new string with sensitive information replaced by placeholders.
    public static func redact(_ string: String) -> String {
        guard let resultPtr = string.withCString({ redact_log($0) }) else {
            return string
        }
        defer { redact_log_free(resultPtr) }
        return String(cString: resultPtr)
    }
}
