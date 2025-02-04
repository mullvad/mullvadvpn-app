//
//  CustomFormatLogHandler.swift
//  MullvadVPN
//
//  Created by pronebird on 02/08/2020.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Logging

public struct CustomFormatLogHandler: LogHandler {
    public var metadata: Logger.Metadata = [:]
    public var logLevel: Logger.Level = .debug

    private let label: String
    private let streams: [TextOutputStream]

    public init(label: String, streams: [TextOutputStream]) {
        self.label = label
        self.streams = streams
    }

    public subscript(metadataKey metadataKey: String) -> Logger.Metadata.Value? {
        get {
            metadata[metadataKey]
        }
        set(newValue) {
            metadata[metadataKey] = newValue
        }
    }

    // swiftlint:disable:next function_parameter_count
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
            .merging(metadata ?? [:]) { _, rhs -> Logger.MetadataValue in
                rhs
            }
        let prettyMetadata = Self.formatMetadata(mergedMetadata)
        let metadataOutput = prettyMetadata.isEmpty ? "" : " \(prettyMetadata)"
        let timestamp = Date().logFormatted
        let formattedMessage = "[\(timestamp)][\(label)][\(level)]\(metadataOutput) \(message)\n"

        for var stream in streams {
            stream.write(formattedMessage)
        }
    }

    private static func formatMetadata(_ metadata: Logger.Metadata) -> String {
        metadata.map { "\($0)=\($1)" }.joined(separator: " ")
    }
}
