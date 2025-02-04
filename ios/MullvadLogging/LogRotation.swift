//
//  LogRotation.swift
//  MullvadVPN
//
//  Created by pronebird on 02/08/2020.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
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
        let storageSizeLimit: Int
        let oldestAllowedDate: Date

        /// Options for log rotation, defining how logs should be retained.
        ///
        /// - Parameter storageSizeLimit: Storage size limit in bytes.
        /// - Parameter oldestAllowedDate: Oldest allowed date.
        public init(storageSizeLimit: Int, oldestAllowedDate: Date) {
            self.storageSizeLimit = storageSizeLimit
            self.oldestAllowedDate = oldestAllowedDate
        }
    }

    public enum Error: LocalizedError, WrappingError {
        case rotateLogFiles(Swift.Error)

        public var errorDescription: String? {
            switch self {
            case .rotateLogFiles:
                return "Failure to rotate logs"
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

            try deleteLogs(dateThreshold: options.oldestAllowedDate, sizeThreshold: options.storageSizeLimit, in: logs)
        } catch {
            throw Error.rotateLogFiles(error)
        }
    }

    private static func deleteLogs(dateThreshold: Date, sizeThreshold: Int, in logs: [LogData]) throws {
        let fileManager = FileManager.default

        var fileSizes = UInt64.zero
        for log in logs {
            fileSizes += log.size

            let logIsTooOld = log.creationDate < dateThreshold
            let logCapacityIsExceeded = fileSizes > sizeThreshold

            if logIsTooOld || logCapacityIsExceeded {
                try fileManager.removeItem(at: log.path)
            }
        }
    }
}
