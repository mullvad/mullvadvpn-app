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

    public static func rotateLogs(logsDirectory: URL) throws {
        let fileManager = FileManager.default

        do {
            // Filter out all log files in directory.
            let logPaths: [URL] = (try fileManager.contentsOfDirectory(
                atPath: logsDirectory.relativePath
            )).compactMap { file in
                if file.split(separator: ".").last == "log" {
                    logsDirectory.appendingPathComponent(file)
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

            // From newest to oldest, delete all logs outside maximum capacity. Currently 5 MB.
            var fileSizes = UInt64.zero
            for log in logs {
                fileSizes += log.size

                if fileSizes > 5_242_880 { // 5 MB
                    try fileManager.removeItem(at: log.path)
                }
            }
        } catch {
            throw Error.rotateLogFiles(error)
        }
    }
}
