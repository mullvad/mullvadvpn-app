//
//  RustLogRedactor.swift
//  MullvadRustRuntime
//
//  Created by Emīls on 2026-01-30.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging

/// Log redactor backed by Rust regex, conforming to `LogRedacting`.
///
/// All state (compiled regexes and container paths) is immutable after construction,
/// making this safe to use from multiple threads without synchronization.
public final class RustLogRedactor: LogRedacting, @unchecked Sendable {
    private let handle: OpaquePointer

    /// Creates a new redactor with predefined redaction rules.
    ///
    /// - Parameters:
    ///   - containerPaths: File system paths whose contents should be redacted
    ///     (for example, app group container paths). Defaults to an empty array.
    ///   - customStrings: Additional strings that should always be redacted.
    public init(containerPaths: [String] = [], customStrings: [String] = []) {
        handle = containerPaths.withCStringArray { containerPathsPtr, containerPathsSize in
            customStrings.withCStringArray { customStringsPtr, customStringsSize in
                create_log_redactor(containerPathsPtr, containerPathsSize, customStringsPtr, customStringsSize)
            }
        }
    }

    deinit {
        log_redactor_free(handle)
    }

    public func redact(_ string: String) -> String {
        guard let resultPtr = string.withCString({ log_redactor_redact(handle, $0) }) else {
            return string
        }
        defer { log_redactor_free_string(resultPtr) }
        return String(cString: resultPtr)
    }
}
