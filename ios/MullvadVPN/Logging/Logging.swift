//
//  Logging.swift
//  MullvadVPN
//
//  Created by pronebird on 02/08/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Logging

func initLoggingSystem(bundleIdentifier: String, metadata: Logger.Metadata? = nil) {
    let containerURL = FileManager.default
        .containerURL(
            forSecurityApplicationGroupIdentifier: ApplicationConfiguration
                .securityGroupIdentifier
        )!
    let logsDirectoryURL = containerURL.appendingPathComponent("Logs", isDirectory: true)
    let logFileName = "\(bundleIdentifier).log"
    let logFileURL = logsDirectoryURL.appendingPathComponent(logFileName)

    // Create Logs folder within container if it doesn't exist
    try? FileManager.default.createDirectory(
        at: logsDirectoryURL,
        withIntermediateDirectories: false,
        attributes: nil
    )

    // Rotate log
    var logRotationError: Error?
    do {
        try LogRotation.rotateLog(
            logsDirectory: logsDirectoryURL,
            logFileName: logFileName
        )
    } catch {
        logRotationError = error
    }

    // Create an array of log output streams
    var streams: [TextOutputStream] = []

    // Create output stream to file
    if let fileLogStream = TextFileOutputStream(fileURL: logFileURL, createFile: true) {
        streams.append(fileLogStream)
    }

    // Configure Logging system
    LoggingSystem.bootstrap { label -> LogHandler in
        var logHandlers: [LogHandler] = []

        #if DEBUG
        logHandlers.append(OSLogHandler(subsystem: bundleIdentifier, category: label))
        #endif

        if !streams.isEmpty {
            logHandlers.append(CustomFormatLogHandler(label: label, streams: streams))
        }

        if logHandlers.isEmpty {
            return SwiftLogNoOpLogHandler()
        } else {
            var multiplex = MultiplexLogHandler(logHandlers)
            if let metadata = metadata {
                multiplex.metadata = metadata
            }
            return multiplex
        }
    }

    if let logRotationError = logRotationError {
        Logger(label: "LogRotation").error(
            error: logRotationError,
            message: "Failed to rotate log"
        )
    }
}
