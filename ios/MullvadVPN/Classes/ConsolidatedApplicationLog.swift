//
//  ConsolidatedApplicationLog.swift
//  MullvadVPN
//
//  Created by pronebird on 29/10/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation

private let kLogMaxReadBytes: UInt64 = 128 * 1024
private let kLogDelimeter = "===================="
private let kRedactedPlaceholder = "[REDACTED]"
private let kRedactedAccountPlaceholder = "[REDACTED ACCOUNT NUMBER]"
private let kRedactedContainerPlaceholder = "[REDACTED CONTAINER PATH]"

class ConsolidatedApplicationLog: TextOutputStreamable {
    typealias Metadata = KeyValuePairs<MetadataKey, String>

    enum MetadataKey: String {
        case id, os
        case productVersion = "mullvad-product-version"
    }

    struct LogAttachment {
        let label: String
        let content: String
    }

    let redactCustomStrings: [String]
    let applicationGroupContainers: [URL]
    let metadata: Metadata

    private var logs: [LogAttachment] = []

    init(
        redactCustomStrings: [String],
        redactContainerPathsForSecurityGroupIdentifiers securityGroupIdentifiers: [String]
    ) {
        metadata = Self.makeMetadata()
        self.redactCustomStrings = redactCustomStrings

        applicationGroupContainers = securityGroupIdentifiers
            .compactMap { securityGroupIdentifier -> URL? in
                return FileManager.default
                    .containerURL(forSecurityApplicationGroupIdentifier: securityGroupIdentifier)
            }
    }

    func addLogFile(fileURL: URL, includeLogBackup: Bool) {
        addSingleLogFile(fileURL)
        if includeLogBackup {
            let oldLogFileURL = fileURL.deletingPathExtension().appendingPathExtension("old.log")
            addSingleLogFile(oldLogFileURL)
        }
    }

    func addLogFiles(fileURLs: [URL], includeLogBackups: Bool) {
        for fileURL in fileURLs {
            addLogFile(fileURL: fileURL, includeLogBackup: includeLogBackups)
        }
    }

    func addError(message: String, error: String) {
        let redactedError = redact(string: error)

        logs.append(LogAttachment(label: message, content: redactedError))
    }

    var string: String {
        var body = ""
        write(to: &body)
        return body
    }

    func write<Target: TextOutputStream>(to stream: inout Target) {
        print("System information:", to: &stream)
        for (key, value) in metadata {
            print("\(key.rawValue): \(value)", to: &stream)
        }
        print("", to: &stream)

        for attachment in logs {
            print(kLogDelimeter, to: &stream)
            print(attachment.label, to: &stream)
            print(kLogDelimeter, to: &stream)
            print(attachment.content, to: &stream)
            print("", to: &stream)
        }
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

        if let lossyString = Self.readFileLossy(path: path, maxBytes: kLogMaxReadBytes) {
            let redactedString = redact(string: lossyString)

            logs.append(LogAttachment(label: redactedPath, content: redactedString))
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

    private static func readFileLossy(path: String, maxBytes: UInt64) -> String? {
        guard let fileHandle = FileHandle(forReadingAtPath: path) else {
            return nil
        }

        let endOfFileOffset = fileHandle.seekToEndOfFile()
        if endOfFileOffset > maxBytes {
            fileHandle.seek(toFileOffset: endOfFileOffset - maxBytes)
        } else {
            fileHandle.seek(toFileOffset: 0)
        }

        let data = fileHandle.readData(ofLength: Int(kLogMaxReadBytes))
        let replacementCharacter = Character(UTF8.decode(UTF8.encodedReplacementCharacter))
        let lossyString = String(
            String(decoding: data, as: UTF8.self)
                .drop { ch in
                    // Drop leading replacement characters produced when decoding data
                    return ch == replacementCharacter
                }
        )

        return lossyString
    }

    private func redactCustomStrings(string: String) -> String {
        return redactCustomStrings.reduce(string) { resultString, redact -> String in
            return resultString.replacingOccurrences(of: redact, with: kRedactedPlaceholder)
        }
    }

    private func redact(string: String) -> String {
        return [
            redactContainerPaths,
            Self.redactAccountNumber,
            Self.redactIPv4Address,
            Self.redactIPv6Address,
            redactCustomStrings,
        ].reduce(string) { resultString, transform -> String in
            return transform(resultString)
        }
    }

    private func redactContainerPaths(string: String) -> String {
        return applicationGroupContainers.reduce(string) { resultString, containerURL -> String in
            return resultString.replacingOccurrences(
                of: containerURL.path,
                with: kRedactedContainerPlaceholder
            )
        }
    }

    private static func redactAccountNumber(string: String) -> String {
        return redact(
            regularExpression: try! NSRegularExpression(pattern: #"\d{16}"#),
            string: string,
            replacementString: kRedactedAccountPlaceholder
        )
    }

    private static func redactIPv4Address(string: String) -> String {
        return redact(
            regularExpression: NSRegularExpression.ipv4RegularExpression,
            string: string,
            replacementString: kRedactedPlaceholder
        )
    }

    private static func redactIPv6Address(string: String) -> String {
        return redact(
            regularExpression: NSRegularExpression.ipv6RegularExpression,
            string: string,
            replacementString: kRedactedPlaceholder
        )
    }

    private static func redact(
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
