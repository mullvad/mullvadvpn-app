//
//  ConsolidatedApplicationLog.swift
//  MullvadVPN
//
//  Created by pronebird on 29/10/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation

private let kLogDelimiter = "===================="
private let kRedactedPlaceholder = "[REDACTED]"
private let kRedactedAccountPlaceholder = "[REDACTED ACCOUNT NUMBER]"
private let kRedactedContainerPlaceholder = "[REDACTED CONTAINER PATH]"

class ConsolidatedApplicationLog: TextOutputStreamable {
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

    let redactCustomStrings: [String]
    let applicationGroupContainers: [URL]
    let metadata: Metadata

    private let logQueue = DispatchQueue(
        label: "com.mullvad.consolidation.logs.queue",
        attributes: .concurrent
    )
    private var logs: [LogAttachment] = []

    init(
        redactCustomStrings: [String],
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

    func addLogFiles(fileURLs: [URL], completion: @escaping () -> Void = {}) {
        logQueue.async(flags: .barrier) {
            for fileURL in fileURLs {
                self.addSingleLogFile(fileURL)
            }
            completion()
        }
    }

    func addError(message: String, error: String, completion: (() -> Void)? = nil) {
        let redactedError = redact(string: error)
        logQueue.async(flags: .barrier) {
            self.logs.append(LogAttachment(label: message, content: redactedError))
            completion?()
        }
    }

    var string: String {
        var result = ""
        logQueue.sync {
            var body = ""
            self.write(to: &body)
            result = body
        }
        return result
    }

    func write(to stream: inout some TextOutputStream) {
        logQueue.sync {
            var localOutput = ""

            localOutput += "System information:\n"
            for (key, value) in metadata {
                localOutput += "\(key.rawValue): \(value)\n"
            }
            localOutput += "\n"
            for attachment in logs {
                localOutput += "\(kLogDelimiter)\n"
                localOutput += "\(attachment.label)\n"
                localOutput += "\(kLogDelimiter)\n"
                localOutput += "\(attachment.content)\n\n"
            }

            stream.write(localOutput)
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

        let data = fileHandle.readData(ofLength: Int(bufferSize))
        let replacementCharacter = Character(UTF8.decode(UTF8.encodedReplacementCharacter))
        let lossyString = String(
            String(decoding: data, as: UTF8.self)
                .drop { ch in
                    // Drop leading replacement characters produced when decoding data
                    ch == replacementCharacter
                }
        )

        return lossyString
    }

    private func redactCustomStrings(string: String) -> String {
        redactCustomStrings.reduce(string) { resultString, redact -> String in
            resultString.replacingOccurrences(of: redact, with: kRedactedPlaceholder)
        }
    }

    private func redact(string: String) -> String {
        [
            redactContainerPaths,
            redactAccountNumber,
            redactIPv4Address,
            redactIPv6Address,
            redactCustomStrings,
        ].reduce(string) { resultString, transform -> String in
            transform(resultString)
        }
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
