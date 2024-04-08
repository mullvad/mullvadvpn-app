//
//  LogRotation.swift
//  MullvadVPN
//
//  Created by pronebird on 02/08/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

public enum LogRotation {
    private struct LogData {
        var path: URL
        var size: UInt64
        var creationDate: Date
    }

    public struct Options {
        let storageSizeLimit: Int?
        let oldestAllowedDate: Date?

        /// Options for log rotation, defining how logs should be retained.
        ///
        /// - Parameter storageSizeLimit: Storage size limit in bytes.
        /// - Parameter oldestAllowedDate: Oldest allowed date.
        public init(storageSizeLimit: Int? = nil, oldestAllowedDate: Date? = nil) {
            self.storageSizeLimit = storageSizeLimit
            self.oldestAllowedDate = oldestAllowedDate
        }
    }

    public enum Error: LocalizedError, WrappingError {
        case rotateLogFiles(Swift.Error)

        public var errorDescription: String? {
            switch self {
            case .rotateLogFiles:
                return "Failure to rotate the source log file to backup."
            }
        }

        public var underlyingError: Swift.Error? {
            switch self {
            case let .rotateLogFiles(error):
                return error
            }
        }
    }

    public static func rotateLogs(logDirectory: URL, options: Options) throws {
        let fileManager = FileManager.default

        do {
            // Filter out all log files in directory.
            let logPaths: [URL] = (try fileManager.contentsOfDirectory(
                atPath: logDirectory.relativePath
            )).compactMap { file in
                if file.split(separator: ".").last == "log" {
                    logDirectory.appendingPathComponent(file)
                } else {
                    nil
                }
            }

            // Convert logs into objects with necessary meta data.
            let logs = try logPaths.map { logPath in
                let attributes = try fileManager.attributesOfItem(atPath: logPath.relativePath)
                let size = (attributes[.size] as? UInt64) ?? 0
                let creationDate = (attributes[.creationDate] as? Date) ?? Date.distantPast

                return LogData(path: logPath, size: size, creationDate: creationDate)
            }.sorted { log1, log2 in
                log1.creationDate > log2.creationDate
            }

            if let oldestAllowedDate = options.oldestAllowedDate {
                try rotateLogsByDate(logs: logs, oldestAllowedDate: oldestAllowedDate)
            }

            if let storageSizeLimit = options.storageSizeLimit {
                try rotateLogsByStorageSizeLimit(logs: logs, storageSizeLimit: storageSizeLimit)
            }
        } catch {
            throw Error.rotateLogFiles(error)
        }
    }

    private static func rotateLogsByDate(logs: [LogData], oldestAllowedDate: Date) throws {
        let fileManager = FileManager.default

        for log in logs where log.creationDate < oldestAllowedDate {
            try fileManager.removeItem(at: log.path)
        }
    }

    private static func rotateLogsByStorageSizeLimit(logs: [LogData], storageSizeLimit: Int) throws {
        let fileManager = FileManager.default

        // From newest to oldest, delete all logs outside maximum capacity.
        var fileSizes = UInt64.zero
        for log in logs {
            fileSizes += log.size

            if fileSizes > storageSizeLimit {
                try fileManager.removeItem(at: log.path)
            }
        }
    }
}
