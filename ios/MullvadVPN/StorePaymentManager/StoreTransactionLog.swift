//
//  StoreTransactionLog.swift
//  MullvadVPN
//
//  Created by pronebird on 26/10/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging

/// Transaction log responsible for storing and querying processed transactions.
///
/// This class is thread safe.
final class StoreTransactionLog {
    private let logger = Logger(label: "StoreTransactionLog")
    private var transactionIdentifiers: Set<String> = []
    private let stateLock = NSLock()

    /// The location of the transaction log file on disk.
    let fileURL: URL

    /// Default location for the transaction log.
    static var defaultFileURL: URL {
        let directories = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask)
        let location = directories.first?.appendingPathComponent("transaction.log", isDirectory: false)
        // swiftlint:disable:next force_unwrapping
        return location!
    }

    /// Default transaction log.
    static let `default` = StoreTransactionLog(fileURL: defaultFileURL)

    /// Initialize the new transaction log.
    ///
    /// - Warning: Panics on attempt to initialize with a non-file URL.
    ///
    /// - Parameter fileURL: a file URL to the transaction log file within the local filesystem.
    init(fileURL: URL) {
        precondition(fileURL.isFileURL, "Only local filesystem URLs are accepted.")
        self.fileURL = fileURL
    }

    /// Check if transaction log contains the transaction identifier.
    ///
    /// - Parameter transactionIdentifier: a transaction identifier.
    /// - Returns: `true` if transaction log contains such transaction identifier, otherwise `false`.
    func contains(transactionIdentifier: String) -> Bool {
        stateLock.withLock {
            transactionIdentifiers.contains(transactionIdentifier)
        }
    }

    /// Add transaction identifier into transaction log.
    ///
    /// Automatically persists the transaction log for new transaction identifiers. Returns immediately If the transaction identifier is already present in the
    /// transaction log.
    ///
    /// - Parameter transactionIdentifier: a transaction identifier.
    func add(transactionIdentifier: String) {
        stateLock.withLock {
            guard !transactionIdentifiers.contains(transactionIdentifier) else { return }

            transactionIdentifiers.insert(transactionIdentifier)
            persist()
        }
    }

    /// Read transaction log from file.
    func read() {
        stateLock.withLock {
            do {
                let serializedString = try String(contentsOf: fileURL)
                transactionIdentifiers = deserialize(from: serializedString)
            } catch {
                switch error {
                case CocoaError.fileReadNoSuchFile, CocoaError.fileNoSuchFile:
                    // Ignore errors pointing at missing transaction log file.
                    break
                default:
                    logger.error(error: error, message: "Failed to load transaction log from disk.")
                }
            }
        }
    }

    /// Persist the transaction identifiers on disk.
    /// Creates the cache directory if it doesn't exist yet.
    private func persist() {
        let serializedData = serialize()

        do {
            try persistInner(serializedString: serializedData)
        } catch CocoaError.fileNoSuchFile {
            createDirectoryAndPersist(serializedString: serializedData)
        } catch {
            logger.error(error: error, message: "Failed to persist transaction log.")
        }
    }

    /// Create the cache directory, then write the transaction log.
    /// - Parameter serializedString: serialized transaction log.
    private func createDirectoryAndPersist(serializedString: String) {
        do {
            try FileManager.default.createDirectory(
                at: fileURL.deletingLastPathComponent(),
                withIntermediateDirectories: true
            )
        } catch {
            logger.error(
                error: error,
                message: "Failed to create a directory for transaction log. Trying to persist once again."
            )
        }

        do {
            try persistInner(serializedString: serializedString)
        } catch {
            logger.error(error: error, message: "Failed to persist transaction log.")
        }
    }

    /// Serialize transaction log into a string.
    /// - Returns: string that contains a serialized transaction log.
    private func serialize() -> String {
        transactionIdentifiers.joined(separator: "\n")
    }

    /// Deserialize transaction log from a string.
    /// - Parameter serializedString: serialized string representation of a transaction log.
    /// - Returns: a set of transaction identifiers.
    private func deserialize(from serializedString: String) -> Set<String> {
        let transactionIdentifiers = serializedString.split { $0.isNewline }
            .map { String($0) }

        return Set(transactionIdentifiers)
    }

    /// Write a list of transaction identifiers on disk.
    /// Transaction identifiers are stored as one per line.
    /// - Parameter serializedString: serialized transaction log
    private func persistInner(serializedString: String) throws {
        try serializedString.write(to: fileURL, atomically: true, encoding: .utf8)
    }
}
