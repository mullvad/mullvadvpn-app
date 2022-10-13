//
//  CustomFormatLogHandler.swift
//  MullvadVPN
//
//  Created by pronebird on 02/08/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Logging

public struct CustomFormatLogHandler: LogHandler {
    public var metadata: Logger.Metadata = [:]
    public var logLevel: Logger.Level = .debug

    private let label: String
    private let streams: [TextOutputStream]

    private let dateFormatter = Self.makeDateFormatter()

    public static func makeDateFormatter() -> DateFormatter {
        let dateFormatter = DateFormatter()
        dateFormatter.dateFormat = "YYYY-MM-dd HH:mm:ss.SSS"
        return dateFormatter
    }

    public init(label: String, streams: [TextOutputStream]) {
        self.label = label
        self.streams = streams
    }

    public subscript(metadataKey metadataKey: String) -> Logger.Metadata.Value? {
        get {
            return metadata[metadataKey]
        }
        set(newValue) {
            metadata[metadataKey] = newValue
        }
    }

    public func log(
        level: Logger.Level,
        message: Logger.Message,
        metadata: Logger.Metadata?,
        source: String,
        file: String,
        function: String,
        line: UInt
    ) {
        let mergedMetadata = self.metadata
            .merging(metadata ?? [:]) { lhs, rhs -> Logger.MetadataValue in
                return rhs
            }
        let prettyMetadata = Self.formatMetadata(mergedMetadata)
        let metadataOutput = prettyMetadata.isEmpty ? "" : " \(prettyMetadata)"
        let timestamp = dateFormatter.string(from: Date())
        let formattedMessage = "[\(timestamp)][\(label)][\(level)]\(metadataOutput) \(message)\n"

        for var stream in streams {
            stream.write(formattedMessage)
        }
    }

    private static func formatMetadata(_ metadata: Logger.Metadata) -> String {
        return metadata.map { "\($0)=\($1)" }.joined(separator: " ")
    }
}
