//
//  LogEntryParser.swift
//  MullvadVPN
//
//  Created by pronebird on 18/08/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

#if DEBUG

import Foundation
import Logging

struct ParsedLogEntry {
    let timestamp: Date
    let level: Logger.Level
    let module: String
    let message: String
}

class LogEntryParser {
    /// Date formatter used for decoding the timestamp
    private let dateFormatter = CustomFormatLogHandler.makeDateFormatter()

    /// Parse a log entry in the following format:
    /// [<DATE>][<MODULE>][<LOG_LEVEL>] <MESSAGE>
    func parse(_ str: String) -> ParsedLogEntry? {
        let ranges = Self.stringRangesWithinSquareBrackets(string: str, maxResults: 3)
        guard ranges.count == 3 else {
            return nil
        }

        let strings = ranges.map { String(str[$0]) }

        guard let timestamp = dateFormatter.date(from: strings[0]),
            let logLevel = Logger.Level(rawValue: strings[2]) else {
                return nil
        }

        // Extract the log message following the log level
        let startIndex = str.index(ranges.last!.upperBound, offsetBy: 1, limitedBy: str.endIndex)
        let message = startIndex.map({ (startIndex) -> String in
            return str[startIndex..<str.endIndex].trimmingCharacters(in: .whitespaces)
        }) ?? ""

        return ParsedLogEntry(
            timestamp: timestamp,
            level: logLevel,
            module: strings[1],
            message: message
        )
    }

    /// Find consecutive ranges of strings within square brackets.
    private static func stringRangesWithinSquareBrackets(string: String, maxResults: Int) -> [Range<String.Index>] {
        var results = [Range<String.Index>]()
        var maybeStartIndex: String.Index?

        guard maxResults > 0 else { return results }

        loop: for (offset, char) in string.enumerated() {
            switch char {
            case "[":
                if maybeStartIndex == nil {
                    maybeStartIndex = string.index(string.startIndex, offsetBy: offset + 1, limitedBy: string.endIndex)
                } else {
                    // out of order
                    break loop
                }

            case "]":
                if let startIndex = maybeStartIndex {
                    maybeStartIndex = nil

                    let endIndex = string.index(string.startIndex, offsetBy: offset)

                    results.append((startIndex..<endIndex))

                    if results.count >= maxResults {
                        // done
                        break loop
                    }
                } else {
                    // out of order
                    break loop
                }

            default:
                continue
            }
        }

        return results
    }

}

#endif
