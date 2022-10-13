//
//  OSLogHandler.swift
//  OSLogHandler
//
//  Created by pronebird on 16/08/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Logging
import os

public struct OSLogHandler: LogHandler {
    public var metadata: Logging.Logger.Metadata = [:]
    public var logLevel: Logging.Logger.Level = .debug

    private let label: String
    private let osLog: OSLog

    private struct RegistryKey: Hashable {
        let subsystem: String
        let category: String
    }

    private static var osLogRegistry: [RegistryKey: OSLog] = [:]
    private static let registryLock = NSLock()

    private static func getOSLog(subsystem: String, category: String) -> OSLog {
        registryLock.lock()
        defer { registryLock.unlock() }

        let key = RegistryKey(subsystem: subsystem, category: category)
        if let log = osLogRegistry[key] {
            return log
        } else {
            let newLog = OSLog(subsystem: subsystem, category: category)
            osLogRegistry[key] = newLog
            return newLog
        }
    }

    public init(subsystem: String, category: String) {
        label = category
        osLog = OSLogHandler.getOSLog(subsystem: subsystem, category: category)
    }

    public subscript(metadataKey metadataKey: String) -> Logging.Logger.Metadata.Value? {
        get {
            return metadata[metadataKey]
        }
        set(newValue) {
            metadata[metadataKey] = newValue
        }
    }

    public func log(
        level: Logging.Logger.Level,
        message: Logging.Logger.Message,
        metadata: Logging.Logger.Metadata?,
        source: String,
        file: String,
        function: String,
        line: UInt
    ) {
        let mergedMetadata = self.metadata
            .merging(metadata ?? [:]) { lhs, rhs -> Logging.Logger.MetadataValue in
                return rhs
            }
        let prettyMetadata = Self.formatMetadata(mergedMetadata)
        let logMessage = prettyMetadata.isEmpty ? message : "\(prettyMetadata) \(message)"

        os_log("%{public}s", log: osLog, type: level.osLogType, "\(logMessage)")
    }

    private static func formatMetadata(_ metadata: Logging.Logger.Metadata) -> String {
        return metadata.map { "\($0)=\($1)" }.joined(separator: " ")
    }
}

private extension Logging.Logger.Level {
    var osLogType: OSLogType {
        switch self {
        case .trace, .debug:
            // Console app does not output .debug logs, use .info instead.
            return .info
        case .info, .notice, .warning:
            return .info
        case .error, .critical:
            return .error
        }
    }
}
