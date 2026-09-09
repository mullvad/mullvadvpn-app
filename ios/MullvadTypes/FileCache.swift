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

/// File cache able to read and write serializable content.
public protocol FileCacheProtocol<Content>: Sendable {
    associatedtype Content: Codable & Sendable

    func read() async throws -> Content
    func write(_ content: Content) async throws
    func clear() async throws

    // Synchronous variants exist only for callers that have not been migrated to async yet.
    // They are removed together with the RelayCache migration.
    @available(*, noasync, message: "Use the async variant from asynchronous contexts")
    func read() throws -> Content
    @available(*, noasync, message: "Use the async variant from asynchronous contexts")
    func write(_ content: Content) throws
    @available(*, noasync, message: "Use the async variant from asynchronous contexts")
    func clear() throws
}

/// File cache actor that can read and write any `Codable` content.
///
/// Cross-process coordination relies on atomic whole-file replacement instead of file locks:
/// writes go to a uniquely named temporary file that is then `rename(2)`d into place. A reader
/// always observes either the previous or the new complete file, never a partial write, and the
/// in-memory cache keyed by file modification time guarantees that content replaced by another
/// process is picked up on the next read.
///
/// The actor's `DispatchSerialQueue` executor means all blocking file I/O and JSON decode/encode
/// happen on a dedicated GCD thread, never on the cooperative pool. Isolated methods never suspend,
/// so synchronous callers can block on the queue through `runBlocking` without a pool thread.
///
/// Multiple `FileCache` instances backed by the same file are safe — writes are atomic and each
/// instance detects external changes through the file modification time. But we should use a shared
/// instance instead. There is no reason for a single file to be backed by multiple file caches in the same process.
public actor FileCache<Content: Codable & Sendable>: FileCacheProtocol, DispatchSerialQueueActor {
    public nonisolated let queue: DispatchSerialQueue

    public nonisolated var unownedExecutor: UnownedSerialExecutor {
        queue.asUnownedSerialExecutor()
    }

    private let fileURL: URL
    private var cachedContent: Content?
    private var contentModified: Date?

    public init(fileURL: URL) {
        self.fileURL = fileURL
        self.queue = DispatchSerialQueue(
            label: "net.mullvad.filecache.\(fileURL.lastPathComponent)",
            qos: .userInitiated
        )
    }

    // MARK: - Asynchronous functions

    public func read() async throws -> Content {
        try readContent()
    }

    public func write(_ content: Content) async throws {
        try writeContent(content)
    }

    public func clear() async throws {
        try clearContent()
    }

    // MARK: - Synchronous functions
    // Only for callers that have not been migrated to async yet.

    @available(*, noasync, message: "Use the async variant from asynchronous contexts")
    public nonisolated func read() throws -> Content {
        try runBlocking { try $0.readContent() }
    }

    @available(*, noasync, message: "Use the async variant from asynchronous contexts")
    public nonisolated func write(_ content: Content) throws {
        try runBlocking { try $0.writeContent(content) }
    }

    @available(*, noasync, message: "Use the async variant from asynchronous contexts")
    public nonisolated func clear() throws {
        try runBlocking { try $0.clearContent() }
    }

    // MARK: - Private

    private func readContent() throws -> Content {
        // Stat before reading, so that a concurrent replacement between the stat and the read can
        // only mark the cache as stale and cause an extra re-read, never serve stale content.
        let modificationTime = Self.fileLastModified(at: fileURL)
        if let cachedContent, let contentModified, modificationTime != nil,
            contentModified == modificationTime
        {
            return cachedContent
        }

        let data = try Data(contentsOf: fileURL)
        let content = try JSONDecoder().decode(Content.self, from: data)

        cachedContent = content
        contentModified = modificationTime

        return content
    }

    private func writeContent(_ content: Content) throws {
        let data = try JSONEncoder().encode(content)
        let tempURL = fileURL.appendingPathExtension("tmp-\(UUID().uuidString)")

        do {
            try data.write(to: tempURL)
            // The rename preserves the modification time. If another process replaces the file
            // afterwards, the stored time no longer matches and the next read hits the disk.
            let writtenModificationTime = Self.fileLastModified(at: tempURL)

            if rename(tempURL.path, fileURL.path) != 0 {
                try? FileManager.default.removeItem(at: tempURL)
                throw FileCacheError.renameFailed(errno)
            }

            cachedContent = content
            contentModified = writtenModificationTime
        } catch {
            cachedContent = nil
            contentModified = nil
            throw error
        }
    }

    private func clearContent() throws {
        cachedContent = nil
        contentModified = nil
        try FileManager.default.removeItem(at: fileURL)
    }

    private static func fileLastModified(at url: URL) -> Date? {
        (try? FileManager.default.attributesOfItem(atPath: url.path))?[.modificationDate] as? Date
    }
}

/// One-shot maintenance for directories containing `FileCache`-backed files, meant to be run once
/// on app launch.
///
/// This code can be removed in 2027.1: by then every install has run a version that no longer
/// creates `.lock` and `.tmp` files, and orphaned `.tmp-<UUID>` files accumulate slowly enough
/// (one small file per process death mid-write) that a year of sweeps is plenty.
public enum FileCacheMaintenance {
    /// Deletes auxiliary files that `FileCache` instances leave behind in `directory`:
    /// `.lock` and `.tmp` files created by versions up to 2026.3, and uniquely named
    /// `.tmp-<UUID>` files orphaned when a process died mid-write. The UUID-named files are
    /// only deleted when older than a day, so that another process's in-flight write is never
    /// swept away.
    ///
    /// Returns the number of deleted orphaned `.tmp-<UUID>` files. Unlike the expected legacy
    /// leftovers, each of those marks a process death mid-write, so callers should log them.
    @discardableResult
    public static func removeStaleCacheFiles(
        in directory: URL,
        olderThan staleAge: TimeInterval = 24 * 60 * 60
    ) -> Int {
        let fileManager = FileManager.default

        guard
            let files = try? fileManager.contentsOfDirectory(
                at: directory,
                includingPropertiesForKeys: [.contentModificationDateKey]
            )
        else { return 0 }

        let staleCutoff = Date(timeIntervalSinceNow: -staleAge)
        var removedOrphanCount = 0

        for url in files {
            let name = url.lastPathComponent

            if name.hasSuffix(".lock") || name.hasSuffix(".tmp") {
                try? fileManager.removeItem(at: url)
            } else if let uuidRange = name.range(of: ".tmp-", options: .backwards),
                UUID(uuidString: String(name[uuidRange.upperBound...])) != nil
            {
                let modified = (try? url.resourceValues(forKeys: [.contentModificationDateKey]))?
                    .contentModificationDate
                if let modified, modified < staleCutoff, (try? fileManager.removeItem(at: url)) != nil {
                    removedOrphanCount += 1
                }
            }
        }

        return removedOrphanCount
    }
}

/// Errors specific to `FileCache` operations.
public enum FileCacheError: LocalizedError {
    /// Atomic rename of temporary file failed.
    case renameFailed(Int32)

    public var errorDescription: String? {
        switch self {
        case let .renameFailed(code):
            return "Failed to rename temporary file: \(String(cString: strerror(code)))"
        }
    }
}
