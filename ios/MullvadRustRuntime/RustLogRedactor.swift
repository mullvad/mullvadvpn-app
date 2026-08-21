// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

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
    public init(containerPaths: [String] = []) {
        handle = containerPaths.withCStringArray { containerPathsPtr, containerPathsSize in
            create_log_redactor(containerPathsPtr, containerPathsSize)
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

    public func addCustomString(_ string: String) {
        guard !string.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty else {
            return
        }
        string.withCString {
            log_redactor_add_custom_string(handle, $0)
        }
    }
}
