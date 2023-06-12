//
//  Logging.swift
//  MullvadVPN
//
//  Created by pronebird on 02/08/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation
@_exported import Logging

private enum LoggerOutput {
    case fileOutput(_ fileOutput: LogFileOutputStream)
    case osLogOutput(_ subsystem: String)
}

public struct LoggerBuilder {
    private(set) var logRotationErrors: [Error] = []
    private var outputs: [LoggerOutput] = []

    public var metadata: Logger.Metadata = [:]
    public var logLevel: Logger.Level = .debug

    public init() {}

    public mutating func addFileOutput(fileURL: URL) {
        let logFileName = fileURL.lastPathComponent
        let logsDirectoryURL = fileURL.deletingLastPathComponent()

        try? FileManager.default.createDirectory(
            at: logsDirectoryURL,
            withIntermediateDirectories: false,
            attributes: nil
        )

        do {
            try LogRotation.rotateLog(logsDirectory: logsDirectoryURL, logFileName: logFileName)
        } catch {
            logRotationErrors.append(error)
        }

        outputs.append(.fileOutput(LogFileOutputStream(fileURL: fileURL)))
    }

    public mutating func addOSLogOutput(subsystem: String) {
        outputs.append(.osLogOutput(subsystem))
    }

    public func install() {
        LoggingSystem.bootstrap { label -> LogHandler in
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
                rotationLogger.error(error: error, message: "Failed to rotate log")
            }
        }
    }
}
