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
public protocol FileCacheProtocol<Content> {
    associatedtype Content: Codable & Sendable

    func read() throws -> Content
    func write(_ content: Content) throws
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
/// The actor's custom `DispatchSerialQueue` executor means all blocking file I/O and JSON
/// decode/encode happen on a single dedicated thread. Isolated methods never suspend, so the
/// synchronous shims never depend on the cooperative thread pool.
///
/// Multiple `FileCache` instances backed by the same file are safe — writes are atomic and each
/// instance detects external changes through the file modification time. But we should use a shared
/// instance instead. There is no reason for a single file to be backed by multiple file caches in the same process.
public actor FileCache<Content: Codable & Sendable>: FileCacheProtocol {
    private enum State {
        case fresh(content: Content, date: Date?)
        case stale
    }

    public nonisolated var unownedExecutor: UnownedSerialExecutor {
        queue.asUnownedSerialExecutor()
    }

    private var cacheState: State = .stale
    private let queue: DispatchSerialQueue
    private let fileURL: URL

    public init(fileURL: URL) {
        self.fileURL = fileURL
        self.queue = DispatchSerialQueue(label: "FileCache")
    }

    // MARK: - Asynchronous functions

    public func read() async throws -> Content {
        switch cacheState {
        case let .fresh(cache, lastModified):
            if lastModified == Self.fileLastModified(at: fileURL) {
                cache
            } else {
                try refresh()
            }
        case .stale:
            try refresh()
        }
    }

    public func write(_ content: Content) async throws {
        let tempURL = fileURL.appendingPathExtension("tmp-\(UUID().uuidString)")

        do {
            let data = try JSONEncoder().encode(content)
            try data.write(to: tempURL)
            let lastModified = Self.fileLastModified(at: tempURL)

            if rename(tempURL.path, fileURL.path) != 0 {
                try? FileManager.default.removeItem(at: tempURL)
                throw FileCacheError.renameFailed(errno)
            }

            cacheState = .fresh(content: content, date: lastModified)
        } catch {
            cacheState = .stale
            throw error
        }
    }

    public func clear() async throws {
        cacheState = .stale
        try FileManager.default.removeItem(at: fileURL)
    }

    // MARK: - Synchronous shims
    // Will be removed once all call sites have been migrated.

    public nonisolated func read() throws -> Content {
        try BridgeExecutor.shared.run {
            try await self.read()
        }
    }

    public nonisolated func write(_ content: Content) throws {
        try BridgeExecutor.shared.run {
            try await self.write(content)
        }
    }

    public nonisolated func clear() throws {
        try BridgeExecutor.shared.run {
            try await self.clear()
        }
    }

    // MARK: - Private

    private func refresh() throws -> Content {
        do {
            let lastModified = Self.fileLastModified(at: fileURL)
            let data = try Data(contentsOf: fileURL)
            let cache = try JSONDecoder().decode(Content.self, from: data)
            cacheState = .fresh(content: cache, date: lastModified)
            return cache
        } catch {
            cacheState = .stale
            throw error
        }
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
