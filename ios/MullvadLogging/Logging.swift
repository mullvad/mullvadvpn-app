//
//  Logging.swift
//  MullvadVPN
//
//  Created by pronebird on 02/08/2020.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
@_exported import Logging
import MullvadTypes

private enum LoggerOutput {
    case fileOutput(_ fileOutput: LogFileOutputStream)
    case osLogOutput(_ subsystem: String)
}

public final class LoggerBuilder: @unchecked Sendable {
    // Static lets are guaranteed by the compiler to be initialized only once in a thread safe way
    public static let shared = LoggerBuilder()

    private init() {
        let pid = ProcessInfo.processInfo.processIdentifier
        metadata = ["pid": .string(String(pid))]
    }

    private static let lock: NSLock = NSLock()

    /// Makes the `install` function idempotent
    static nonisolated(unsafe) private var initializedLoggingSystem = false

    private var logRotationErrors: [Error] = []
    private var outputs: [LoggerOutput] = []

    private let metadata: Logger.Metadata
    private var logLevel: Logger.Level = .debug
    private var header: String = ""

    public func setHeader(_ newHeader: String) {
        Self.lock.withLock {
            Self.shared.header = newHeader
        }
    }

    public func addFileOutput(fileURL: URL) {
        Self.lock.withLock {
            let logsDirectoryURL = fileURL.deletingLastPathComponent()

            try? FileManager.default.createDirectory(
                at: logsDirectoryURL,
                withIntermediateDirectories: false,
                attributes: nil
            )

            do {
                try LogRotation.rotateLogs(
                    logDirectory: logsDirectoryURL,
                    options: LogRotation.Options(
                        storageSizeLimit: 2_000_000,  // 2 MB
                        oldestAllowedDate: Date(timeIntervalSinceNow: -Duration.days(7).timeInterval)
                    ))
            } catch {
                logRotationErrors.append(error)
            }

            outputs.append(.fileOutput(LogFileOutputStream(fileURL: fileURL, header: header)))
        }
    }

    public func addOSLogOutput(subsystem: String) {
        Self.lock.withLock {
            outputs.append(.osLogOutput(subsystem))
        }
    }

    public func install() {
        Self.lock.withLock {
            guard Self.initializedLoggingSystem == false else { return }
            Self.initializedLoggingSystem = true

            LoggingSystem.bootstrap { [self] label -> LogHandler in
                let logHandlers: [LogHandler] = outputs.map { output in
                    switch output {
                    case let .fileOutput(stream):
                        return CustomFormatLogHandler(label: label, streams: [stream])

                    case let .osLogOutput(subsystem):
                        return OSLogHandler(subsystem: subsystem, category: label)
                    }
                }

                if logHandlers.isEmpty {
                    return SwiftLogNoOpLogHandler()
                } else {
                    var multiplex = MultiplexLogHandler(logHandlers)
                    multiplex.metadata = metadata
                    multiplex.logLevel = logLevel
                    return multiplex
                }
            }

            if !logRotationErrors.isEmpty {
                let rotationLogger = Logger(label: "LogRotation")

                for error in logRotationErrors {
                    rotationLogger.error(error: error, message: error.localizedDescription)
                }
            }
        }
    }
}
