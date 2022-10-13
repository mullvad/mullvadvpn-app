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
    public enum Error: LocalizedError, WrappingError {
        case noSourceLogFile
        case moveSourceLogFile(Swift.Error)

        public var errorDescription: String? {
            switch self {
            case .noSourceLogFile:
                return "Source log file does not exist."
            case .moveSourceLogFile:
                return "Failure to move the source log file to backup."
            }
        }

        public var underlyingError: Swift.Error? {
            switch self {
            case .noSourceLogFile:
                return nil
            case let .moveSourceLogFile(error):
                return error
            }
        }
    }

    public static func rotateLog(logsDirectory: URL, logFileName: String) throws {
        let source = logsDirectory.appendingPathComponent(logFileName)
        let backup = source.deletingPathExtension().appendingPathExtension("old.log")

        do {
            _ = try FileManager.default.replaceItemAt(backup, withItemAt: source)
        } catch {
            // FileManager returns a very obscure error chain so we need to traverse it to find
            // the root cause of the error.
            for case let fileError as CocoaError in error.underlyingErrorChain {
                // .fileNoSuchFile is returned when both backup and source log files do not exist
                // .fileReadNoSuchFile is returned when backup exists but source log file does not
                if fileError.code == .fileNoSuchFile || fileError.code == .fileReadNoSuchFile,
                   fileError.url == source
                {
                    throw Error.noSourceLogFile
                }
            }

            throw Error.moveSourceLogFile(error)
        }
    }
}
