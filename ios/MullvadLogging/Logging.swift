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

public struct LoggerBuilder {
    private(set) var logRotationErrors: [Error] = []
    private var outputs: [LoggerOutput] = []
    private let _swiftLogBootstrapGuard = SwiftLogBootstrap.shared

    public var metadata: Logger.Metadata = [:]
    public var logLevel: Logger.Level = .debug
    public var header: String

    public init(header: String) {
        self.header = header
    }

    public mutating func addFileOutput(fileURL: URL) {
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

    public mutating func addOSLogOutput(subsystem: String) {
        outputs.append(.osLogOutput(subsystem))
    }

    public func install() {
        let didInstall = _swiftLogBootstrapGuard.installIfNeeded { label -> LogHandler in
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
        guard didInstall else { return }

        if !logRotationErrors.isEmpty {
            let rotationLogger = Logger(label: "LogRotation")

            for error in logRotationErrors {
                rotationLogger.error(error: error, message: error.localizedDescription)
            }
        }
    }
}

private final class SwiftLogBootstrap: @unchecked Sendable {
    static let shared = SwiftLogBootstrap()

    private let lock = NSLock()
    private var installed = false

    func installIfNeeded(_ factory: @escaping (String) -> LogHandler) -> Bool {
        lock.lock()
        defer { lock.unlock() }
        if installed { return false }
        installed = true
        LoggingSystem.bootstrap(factory)
        return true
    }
}
