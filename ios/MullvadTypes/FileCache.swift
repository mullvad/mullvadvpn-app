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
/// Uses `flock()` for cross-process synchronization (shared locks for reads, exclusive locks for writes)
/// and an in-memory cache keyed by file modification time to skip I/O when the file hasn't changed.
///
/// Writes are crash-safe: data is written to a temporary file first, then atomically renamed into place
/// while holding the exclusive lock. If a process crashes mid-write, the original file remains intact
/// and the lock is automatically released by the kernel when the file descriptor is closed.
///
/// A separate lock file (`.lock` extension) is used as the locking point so that atomic renames
/// don't invalidate held locks.
///
/// Multiple `FileCache` instances backed by the same file are safe — they all coordinate through the
/// same lock file.
public final class FileCache<Content: Codable>: FileCacheProtocol, @unchecked Sendable {
    public let fileURL: URL
    private let lockFileURL: URL

    /// Serial queue protecting `cachedContent` and `cachedMtime` against data races.
    private let cacheQueue = DispatchQueue(label: "net.mullvadvpn.FileCache.cacheQueue")
    private var cachedContent: Content?
    private var cachedMtime: Date?

    public init(fileURL: URL) {
        self.fileURL = fileURL
        self.lockFileURL = fileURL.appendingPathExtension("lock")
    }

    public func read() throws -> Content {
        // Fast path: if the file modification time hasn't changed, return the cached content.
        let currentMtime = fileMtime()
        let cached: Content? = cacheQueue.sync {
            if let cachedContent, let cachedMtime, cachedMtime == currentMtime, currentMtime != nil {
                return cachedContent
            }
            return nil
        }
        if let cached {
            return cached
        }

        // Slow path: acquire a shared lock and read from disk.
        let lockFd = try openLockFile()
        defer { close(lockFd) }

        guard flock(lockFd, LOCK_SH) == 0 else {
            throw FileCacheError.lockFailed(errno)
        }
        defer { flock(lockFd, LOCK_UN) }

        let data = try Data(contentsOf: fileURL)
        let content = try JSONDecoder().decode(Content.self, from: data)
        let mtime = fileMtime()

        cacheQueue.sync {
            cachedContent = content
            cachedMtime = mtime
        }

        return content
    }

    public func write(_ content: Content) throws {
        let lockFd = try openLockFile()
        defer { close(lockFd) }

        guard flock(lockFd, LOCK_EX) == 0 else {
            throw FileCacheError.lockFailed(errno)
        }
        defer { flock(lockFd, LOCK_UN) }

        // Write to a temporary file first, then atomically rename for crash safety.
        let tempURL = fileURL.appendingPathExtension("tmp")
        let data = try JSONEncoder().encode(content)
        try data.write(to: tempURL)

        if rename(tempURL.path, fileURL.path) != 0 {
            // Clean up temp file on failure.
            try? FileManager.default.removeItem(at: tempURL)
            throw FileCacheError.renameFailed(errno)
        }

        let mtime = fileMtime()
        cacheQueue.sync {
            cachedContent = content
            cachedMtime = mtime
        }
    }

    public func clear() throws {
        let lockFd = try openLockFile()
        defer { close(lockFd) }

        guard flock(lockFd, LOCK_EX) == 0 else {
            throw FileCacheError.lockFailed(errno)
        }
        defer { flock(lockFd, LOCK_UN) }

        try FileManager.default.removeItem(at: fileURL)

        cacheQueue.sync {
            cachedContent = nil
            cachedMtime = nil
        }
    }

    // MARK: - Private

    private func openLockFile() throws -> Int32 {
        let fd = open(lockFileURL.path, O_RDWR | O_CREAT, 0o644)
        guard fd >= 0 else {
            throw FileCacheError.openFailed(errno)
        }
        return fd
    }

    private func fileMtime() -> Date? {
        (try? FileManager.default.attributesOfItem(atPath: fileURL.path))?[.modificationDate] as? Date
    }
}

/// Errors specific to `FileCache` operations.
public enum FileCacheError: LocalizedError {
    /// `flock()` failed with the given `errno`.
    case lockFailed(Int32)
    /// Could not open or create the lock file.
    case openFailed(Int32)
    /// Atomic rename of temporary file failed.
    case renameFailed(Int32)

    public var errorDescription: String? {
        switch self {
        case let .lockFailed(code):
            return "Failed to acquire file lock: \(String(cString: strerror(code)))"
        case let .openFailed(code):
            return "Failed to open lock file: \(String(cString: strerror(code)))"
        case let .renameFailed(code):
            return "Failed to rename temporary file: \(String(cString: strerror(code)))"
        }
    }
}
