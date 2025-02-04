//
//  ConsolidatedApplicationLog.swift
//  MullvadVPN
//
//  Created by pronebird on 29/10/2020.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation

private let kLogDelimiter = "===================="
private let kRedactedPlaceholder = "[REDACTED]"
private let kRedactedAccountPlaceholder = "[REDACTED ACCOUNT NUMBER]"
private let kRedactedContainerPlaceholder = "[REDACTED CONTAINER PATH]"

class ConsolidatedApplicationLog: TextOutputStreamable, @unchecked Sendable {
    typealias Metadata = KeyValuePairs<MetadataKey, String>
    private let bufferSize: UInt64

    enum MetadataKey: String {
        case id, os
        case productVersion = "mullvad-product-version"
    }

    struct LogAttachment {
        let label: String
        let content: String
    }

    let redactCustomStrings: [String]?
    let applicationGroupContainers: [URL]
    let metadata: Metadata

    private let logQueue = DispatchQueue(label: "com.mullvad.consolidation.logs.queue")
    private var logs: [LogAttachment] = []

    init(
        redactCustomStrings: [String]? = nil,
        redactContainerPathsForSecurityGroupIdentifiers securityGroupIdentifiers: [String],
        bufferSize: UInt64
    ) {
        metadata = Self.makeMetadata()
        self.redactCustomStrings = redactCustomStrings
        self.bufferSize = bufferSize

        applicationGroupContainers = securityGroupIdentifiers
            .compactMap { securityGroupIdentifier -> URL? in
                FileManager.default
                    .containerURL(forSecurityApplicationGroupIdentifier: securityGroupIdentifier)
            }
    }

    func addLogFiles(fileURLs: [URL], completion: (@Sendable () -> Void)? = nil) {
        logQueue.async(flags: .barrier) {
            for fileURL in fileURLs {
                self.addSingleLogFile(fileURL)
            }
            DispatchQueue.main.async {
                completion?()
            }
        }
    }

    func addError(message: String, error: String, completion: (@Sendable () -> Void)? = nil) {
        let redactedError = redact(string: error)
        logQueue.async(flags: .barrier) {
            self.logs.append(LogAttachment(label: message, content: redactedError))
            DispatchQueue.main.async {
                completion?()
            }
        }
    }

    var string: String {
        var logsCopy: [LogAttachment] = []
        var metadataCopy: Metadata = [:]
        logQueue.sync {
            logsCopy = logs
            metadataCopy = metadata
        }
        guard !logsCopy.isEmpty else { return "" }
        return formatLog(logs: logsCopy, metadata: metadataCopy)
    }

    func write(to stream: inout some TextOutputStream) {
        var logsCopy: [LogAttachment] = []
        var metadataCopy: Metadata = [:]
        logQueue.sync {
            logsCopy = logs
            metadataCopy = metadata
        }
        let localOutput = formatLog(logs: logsCopy, metadata: metadataCopy)
        stream.write(localOutput)
    }

    private func formatLog(logs: [LogAttachment], metadata: Metadata) -> String {
        var result = "System information:\n"
        for (key, value) in metadata {
            result += "\(key.rawValue): \(value)\n"
        }
        result += "\n"
        for attachment in logs {
            result += "\(kLogDelimiter)\n"
            result += "\(attachment.label)\n"
            result += "\(kLogDelimiter)\n"
            result += "\(attachment.content)\n\n"
        }
        return result
    }

    private func addSingleLogFile(_ fileURL: URL) {
        guard fileURL.isFileURL else {
            addError(
                message: fileURL.absoluteString,
                error: "Invalid log file URL: \(fileURL.absoluteString)."
            )
            return
        }

        let path = fileURL.path
        let redactedPath = redact(string: path)

        if let lossyString = readFileLossy(path: path, maxBytes: bufferSize) {
            let redactedString = redact(string: lossyString)
            logQueue.async(flags: .barrier) {
                self.logs.append(LogAttachment(label: redactedPath, content: redactedString))
            }
        } else {
            addError(message: redactedPath, error: "Log file does not exist: \(path).")
        }
    }

    private static func makeMetadata() -> Metadata {
        let osVersion = ProcessInfo.processInfo.operatingSystemVersion
        let osVersionString =
            "iOS \(osVersion.majorVersion).\(osVersion.minorVersion).\(osVersion.patchVersion)"

        return [
            .id: UUID().uuidString,
            .productVersion: Bundle.main.productVersion,
            .os: osVersionString,
        ]
    }

    private func readFileLossy(path: String, maxBytes: UInt64) -> String? {
        guard let fileHandle = FileHandle(forReadingAtPath: path) else {
            return nil
        }

        let endOfFileOffset = fileHandle.seekToEndOfFile()
        if endOfFileOffset > maxBytes {
            fileHandle.seek(toFileOffset: endOfFileOffset - maxBytes)
        } else {
            fileHandle.seek(toFileOffset: 0)
        }

        let replacementCharacter = Character(UTF8.decode(UTF8.encodedReplacementCharacter))
        if let data = try? fileHandle.read(upToCount: Int(bufferSize)),
           let lossyString = String(bytes: data, encoding: .utf8) {
            let resultString = lossyString.drop { ch in
                // Drop leading replacement characters produced when decoding data
                ch == replacementCharacter
            }
            return String(resultString)
        } else {
            return nil
        }
    }

    private func redactCustomStrings(in string: String) -> String {
        guard let customStrings = redactCustomStrings,
              !customStrings.isEmpty else {
            return string
        }
        return customStrings.reduce(string) { resultString, redact in
            resultString.replacingOccurrences(of: redact, with: kRedactedPlaceholder)
        }
    }

    private func redact(string: String) -> String {
        var result = string
        result = redactContainerPaths(string: result)
        result = redactAccountNumber(string: result)
        result = redactIPv4Address(string: result)
        result = redactIPv6Address(string: result)
        result = redactCustomStrings(in: result)
        return result
    }

    private func redactContainerPaths(string: String) -> String {
        applicationGroupContainers.reduce(string) { resultString, containerURL -> String in
            resultString.replacingOccurrences(
                of: containerURL.path,
                with: kRedactedContainerPlaceholder
            )
        }
    }

    private func redactAccountNumber(string: String) -> String {
        redact(
            // swiftlint:disable:next force_try
            regularExpression: try! NSRegularExpression(pattern: #"\d{16}"#),
            string: string,
            replacementString: kRedactedAccountPlaceholder
        )
    }

    private func redactIPv4Address(string: String) -> String {
        redact(
            regularExpression: NSRegularExpression.ipv4RegularExpression,
            string: string,
            replacementString: kRedactedPlaceholder
        )
    }

    private func redactIPv6Address(string: String) -> String {
        redact(
            regularExpression: NSRegularExpression.ipv6RegularExpression,
            string: string,
            replacementString: kRedactedPlaceholder
        )
    }

    private func redact(
        regularExpression: NSRegularExpression,
        string: String,
        replacementString: String
    ) -> String {
        let nsRange = NSRange(string.startIndex ..< string.endIndex, in: string)
        let template = NSRegularExpression.escapedTemplate(for: replacementString)

        return regularExpression.stringByReplacingMatches(
            in: string,
            options: [],
            range: nsRange,
            withTemplate: template
        )
    }
}
