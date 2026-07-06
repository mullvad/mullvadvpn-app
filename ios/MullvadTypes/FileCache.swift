//
//  FileCache.swift
//  MullvadTypes
//
//  Created by Marco Nikic on 2023-05-30.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Protocol describing file cache that's able to read and write serializable content.
public protocol FileCacheProtocol<Content> {
    associatedtype Content: Codable

    func read() throws -> Content
    func write(_ content: Content) throws
    func clear() throws
}

/// File cache implementation that can read and write any `Codable` content.
///
/// Cross-process coordination relies on atomic whole-file replacement instead of file locks:
/// writes go to a uniquely named temporary file that is then `rename(2)`d into place. A reader
/// always observes either the previous or the new complete file, never a partial write, and the
/// in-memory cache keyed by file modification time guarantees that content replaced by another
/// process is picked up on the next read.
///
/// Multiple `FileCache` instances backed by the same file are safe — writes are atomic and each
/// instance detects external changes through the file modification time. But we should use a shared
/// instance instead. There is no reason for a single file to be backed by multiple file caches in the same process.
public final class FileCache<Content: Codable>: FileCacheProtocol, @unchecked Sendable {
    public let fileURL: URL

    /// Lock protecting `cachedContent` and `contentModified` against data races.
    private let cacheLock = NSLock()
    private var cachedContent: Content?
    private var contentModified: Date?

    public init(fileURL: URL) {
        self.fileURL = fileURL
    }

    public func read() throws -> Content {
        cacheLock.lock()
        defer { cacheLock.unlock() }

        // Stat before reading, so that a concurrent replacement between the stat and the read can
        // only mark the cache as stale and cause an extra re-read, never serve stale content.
        let modificationTime = fileModificationTime(at: fileURL)
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

    public func write(_ content: Content) throws {
        cacheLock.lock()
        defer { cacheLock.unlock() }

        let data = try JSONEncoder().encode(content)

        // Write to a uniquely named temporary file, then atomically rename into place. The unique
        // name prevents concurrent writers in this or another process from clobbering each other's
        // in-progress writes.
        let tempURL = fileURL.appendingPathExtension("tmp-\(UUID().uuidString)")
        try data.write(to: tempURL)

        // Capture the modification time before the rename, which preserves it. If another process
        // replaces the file afterwards, the stored time no longer matches and the next read
        // re-reads from disk.
        let writtenModificationTime = fileModificationTime(at: tempURL)

        if rename(tempURL.path, fileURL.path) != 0 {
            try? FileManager.default.removeItem(at: tempURL)
            throw FileCacheError.renameFailed(errno)
        }

        cachedContent = content
        contentModified = writtenModificationTime
    }

    public func clear() throws {
        cacheLock.lock()
        defer { cacheLock.unlock() }

        try FileManager.default.removeItem(at: fileURL)

        cachedContent = nil
        contentModified = nil
    }

    // MARK: - Private

    private func fileModificationTime(at url: URL) -> Date? {
        (try? FileManager.default.attributesOfItem(atPath: url.path))?[.modificationDate] as? Date
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
