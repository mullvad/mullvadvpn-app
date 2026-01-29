//
//  RustLogging.swift
//  MullvadRustRuntime
//
//  Created by Emīls on 2026-01-09.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging

/// Box holding a Logger instance so it can be passed as an opaque pointer through Rust FFI.
private final class LoggerBox {
    let logger: Logger

    init(_ logger: Logger) {
        self.logger = logger
    }
}

/// C-compatible logging callback function.
/// This must be a global function (not a closure) to be passed as a C function pointer.
///
/// The `context` parameter is an opaque pointer to a `LoggerBox` that travels through Rust
/// and back, so this callback doesn't rely on any global state.
///
/// Thread safety: this callback may be invoked concurrently from any thread.
private func rustLogCallback(
    context: UnsafeMutableRawPointer?,
    level: UInt8,
    targetPtr: UnsafePointer<CChar>?,
    messagePtr: UnsafePointer<CChar>?
) {
    guard let context, let targetPtr, let messagePtr else { return }

    let box = Unmanaged<LoggerBox>.fromOpaque(context).takeUnretainedValue()
    let target = String(cString: targetPtr)
    let message = String(cString: messagePtr)

    let logLevel: Logger.Level =
        switch level {
        case 1:
            .error
        case 2:
            .warning
        case 3:
            .info
        case 4:
            .debug
        case 5:
            .trace
        default:
            .debug
        }

    box.logger.log(level: logLevel, "\(message)", metadata: ["rust": .string(target)])
}

/// Initializes the Rust logging system to forward logs to Swift's Logger.
///
/// This function should be called once early in the application lifecycle,
/// before any Rust code that uses logging is invoked.
public enum RustLogging {
    /// Initialize Rust logging to forward to the given Swift Logger.
    /// Safe to call multiple times - only the first call has effect.
    public static func initialize(logger: Logger) {
        let box = LoggerBox(logger)
        let context = Unmanaged.passRetained(box).toOpaque()
        init_rust_logging(rustLogCallback, context)
    }
}
