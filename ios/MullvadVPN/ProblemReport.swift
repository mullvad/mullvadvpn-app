//
//  ProblemReport.swift
//  MullvadVPN
//
//  Created by pronebird on 29/10/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation

private let kLogMaxReadBytes: UInt64 = 128 * 1024
private let kLogDelimeter = "===================="

class ProblemReport {

    struct LogFileAttachment {
        let label: String
        let content: String
    }

    enum Error: ChainedError {
        case logFileDoesNotExist(String)

        var errorDescription: String? {
            switch self {
            case .logFileDoesNotExist(let path):
                return "Cannot read the log file: \(path)"
            }
        }
    }

    enum MetadataKey: String {
        case id, os
        case productVersion = "mullvad-product-version"
    }

    typealias Metadata = KeyValuePairs<MetadataKey, String>

//    private let metadata: Metadata
    private let redactCustomStrings: [String]
    private var metadata: Metadata = [:]
    private var logs: [LogFileAttachment] = []

    init(redactCustomStrings: [String]) {
        self.metadata = Self.makeMetadata()
        self.redactCustomStrings = redactCustomStrings
    }

    func addLog(filePath: String) {
        switch self.readFileLossy(path: filePath, maxBytes: kLogMaxReadBytes) {
        case .success(let lossyString):
            let redactedString = redact(input: lossyString)
            logs.append(LogFileAttachment(label: filePath, content: redactedString))
        case .failure(let error):
            addError(message: filePath, error: error)
        }
    }

    func addError<ErrorType: ChainedError>(message: String, error: ErrorType) {
        let redactedError = redact(input: error.displayChain())

        logs.append(LogFileAttachment(label: message, content: redactedError))
    }

    func write<Target: TextOutputStream>(into stream: inout Target) {
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

    private static func makeMetadata() -> Metadata {
        let bundle = Bundle(for: ProblemReport.self)
        let version = bundle.object(forInfoDictionaryKey: kCFBundleVersionKey as String) as? String ?? "nil"
        let shortVersion = bundle.object(forInfoDictionaryKey: "CFBundleShortVersionString") as? String ?? "nil"

        let operatingSystemVersion: String = {
            let version = ProcessInfo.processInfo.operatingSystemVersion
            return "iOS \(version.majorVersion).\(version.minorVersion).\(version.patchVersion)"
        }()

        return [
            .id : UUID().uuidString,
            .productVersion: "\(version)-b\(shortVersion)",
            .os: operatingSystemVersion
        ]
    }

    private func readFileLossy(path: String, maxBytes: UInt64) -> Result<String, Error> {
        guard let fileHandle = FileHandle(forReadingAtPath: path) else {
            return .failure(.logFileDoesNotExist(path))
        }

        let endOfFileOffset = fileHandle.seekToEndOfFile()
        if endOfFileOffset > maxBytes {
            fileHandle.seek(toFileOffset: endOfFileOffset - maxBytes)
        } else {
            fileHandle.seek(toFileOffset: 0)
        }

        let data = fileHandle.readData(ofLength: Int(kLogMaxReadBytes))
        let lossyString = String(decoding: data, as: UTF8.self)

        return .success(lossyString)
    }

    private func redact(input: String) -> String {
        return [self.redactAccountNumber, self.redactCustomStrings]
            .reduce(input) { (accum, transform) -> String in
                return transform(accum)
            }
    }

    private func redactAccountNumber(input: String) -> String {
        let regex = try! NSRegularExpression(pattern: #"\d{16}"#, options: [])
        let nsRange = NSRange((input.startIndex..<input.endIndex), in: input)

        return regex.stringByReplacingMatches(
            in: input,
            options: [],
            range: nsRange,
            withTemplate: "[REDACTED ACCOUNT NUMBER]"
        )
    }

    private func redactCustomStrings(input: String) -> String {
        return redactCustomStrings.reduce(input) { (input, redact) -> String in
            return input.replacingOccurrences(of: redact, with: "[REDACTED]")
        }
    }

}
