// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation
import Logging

public struct CustomFormatLogHandler: @unchecked Sendable, LogHandler {
    public var metadata: Logger.Metadata = [:]
    public var logLevel: Logger.Level = .debug

    private let label: String
    private let streams: [TextOutputStream]
    private let redactor: LogRedacting

    public init(label: String, streams: [TextOutputStream], redactor: LogRedacting) {
        self.label = label
        self.streams = streams
        self.redactor = redactor
    }

    public subscript(metadataKey metadataKey: String) -> Logger.Metadata.Value? {
        get {
            metadata[metadataKey]
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
            .merging(metadata ?? [:]) { _, rhs -> Logger.MetadataValue in
                rhs
            }
        let prettyMetadata = Self.formatMetadata(mergedMetadata)
        let metadataOutput = prettyMetadata.isEmpty ? "" : " \(prettyMetadata)"
        let timestamp = Date().logFormatted
        let redactedMessage = redactor.redact(message.description)
        let formattedMessage = "[\(timestamp)][\(label)][\(level)]\(metadataOutput) \(redactedMessage)\n"

        for var stream in streams {
            stream.write(formattedMessage)
        }
    }

    private static func formatMetadata(_ metadata: Logger.Metadata) -> String {
        metadata.map { "\($0)=\($1)" }.joined(separator: " ")
    }
}
