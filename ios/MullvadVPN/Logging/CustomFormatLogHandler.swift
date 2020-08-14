//
//  CustomFormatLogHandler.swift
//  MullvadVPN
//
//  Created by pronebird on 02/08/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Logging

struct CustomFormatLogHandler: LogHandler {
    var metadata: Logger.Metadata = [:]
    var logLevel: Logger.Level = .debug

    private let label: String
    private let streams: [TextOutputStream]

    init(label: String, streams: [TextOutputStream]) {
        self.label = label
        self.streams = streams
    }

    subscript(metadataKey metadataKey: String) -> Logger.Metadata.Value? {
        get {
            return metadata[metadataKey]
        }
        set(newValue) {
            metadata[metadataKey] = newValue
        }
    }

    func log(level: Logger.Level,
             message: Logger.Message,
             metadata: Logger.Metadata?,
             source: String,
             file: String,
             function: String,
             line: UInt)
    {
        let mergedMetadata = self.metadata.merging(metadata ?? [:]) { (lhs, rhs) -> Logger.MetadataValue in
            return rhs
        }
        let prettyMetadata = Self.formatMetadata(mergedMetadata)
        let metadataOutput = prettyMetadata.isEmpty ? "" :  " \(prettyMetadata)"
        let formattedMessage = "[\(Self.timestamp())][\(self.label)][\(level)]\(metadataOutput) \(message)\n"

        for var stream in streams {
            stream.write(formattedMessage)
        }
    }

    private static func formatMetadata(_ metadata: Logger.Metadata) -> String {
        return metadata.map { "\($0)=\($1)" }.joined(separator: " ")
    }

    private static func timestamp() -> String {
        var buffer = [Int8](repeating: 0, count: 255)
        var timestamp = time(nil)
        let localTime = localtime(&timestamp)
        strftime(&buffer, buffer.count, "%Y-%m-%dT%H:%M:%S%z", localTime)
        return buffer.withUnsafeBufferPointer {
            $0.withMemoryRebound(to: CChar.self) {
                String(cString: $0.baseAddress!)
            }
        }
    }
}
