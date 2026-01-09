//
//  RustLogging.swift
//  MullvadRustRuntime
//
//  Created by Emīls on 2026-01-09.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging

/// Global logger instance for Rust logs
private let rustLogger = Logger(label: "Rust")

/// C-compatible logging callback function.
/// This must be a global function (not a closure) to be passed as a C function pointer.
private func rustLogCallback(level: UInt8, messagePtr: UnsafePointer<CChar>?) {
    guard let messagePtr else { return }
    let message = String(cString: messagePtr)

    let level: Logger.Level = switch level {
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

    rustLogger.log(level: level, "\(message)")
}

/// Initializes the Rust logging system to forward logs to Swift's Logger.
///
/// This function should be called once early in the application lifecycle,
/// before any Rust code that uses logging is invoked.
public enum RustLogging {
    /// Initialize Rust logging to forward to Swift Logger.
    /// Safe to call multiple times - only the first call has effect.
    public static func initialize() {
        init_rust_logging(rustLogCallback)
    }
}
