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

    /// Creates a new redactor with the given container paths baked in.
    ///
    /// - Parameter containerPaths: File system paths to redact (e.g. app group container paths).
    public init(containerPaths: [String] = []) {
        let cPaths = containerPaths.map { strdup($0)! }
        defer { cPaths.forEach { free($0) } }

        let pathPtrs: [UnsafePointer<CChar>?] = cPaths.map { UnsafePointer($0) }
        handle = pathPtrs.withUnsafeBufferPointer { buffer in
            create_log_redactor(buffer.baseAddress, UInt(buffer.count))
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
