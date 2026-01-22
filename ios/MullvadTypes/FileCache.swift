//
//  FileCache.swift
//  MullvadTypes
//
//  Created by Marco Nikic on 2023-05-30.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Protocol describing file cache that's able to read and write serializable content.
public protocol FileCacheProtocol<Content> {
    associatedtype Content: Codable

    func read() throws -> Content
    func write(_ content: Content) throws
    func clear() throws
}

/// File cache implementation that can read and write any `Codable` content and uses file coordinator to coordinate I/O.
/// Caches content in memory after initial load and invalidates the cache when file changes are detected from other processes.
public final class FileCache<Content: Codable>: NSObject, FileCacheProtocol, NSFilePresenter, @unchecked Sendable {
    public let fileURL: URL
    public var presentedItemURL: URL? { fileURL }
    public let presentedItemOperationQueue = OperationQueue()

    /// In-memory cache of the content.
    private var cachedContent: Content?

    /// Lock to synchronize access to the in-memory cache.
    private let lock = NSLock()

    public init(fileURL: URL) {
        self.fileURL = fileURL
        super.init()
        presentedItemOperationQueue.maxConcurrentOperationCount = 1

        // Register as file presenter to receive change notifications
        NSFileCoordinator.addFilePresenter(self)

        // Load initial content off the main thread
        DispatchQueue(label: "net.mullvadvpn.FileCache.\(fileURL)").async {
            self.loadInitialCache()
        }
    }

    deinit {
        NSFileCoordinator.removeFilePresenter(self)
    }

    /// Loads the initial cache from disk. Called during initialization.
    /// The lock ensures any concurrent read() calls block until loading completes.
    private func loadInitialCache() {
        lock.lock()
        defer { lock.unlock() }

        cachedContent = try? readFromDisk()
    }

    public func read() throws -> Content {
        lock.lock()
        defer { lock.unlock() }

        if let cached = cachedContent {
            return cached
        }

        // If cache is nil (e.g., initial load failed or was invalidated), read from disk
        let content = try readFromDisk()
        cachedContent = content
        return content
    }

    /// Reads content directly from disk using file coordination.
    private func readFromDisk() throws -> Content {
        let fileCoordinator = NSFileCoordinator(filePresenter: self)

        return try fileCoordinator.coordinate(readingItemAt: fileURL, options: [.withoutChanges]) { fileURL in
            try JSONDecoder().decode(Content.self, from: Data(contentsOf: fileURL))
        }
    }

    public func write(_ content: Content) throws {
        lock.lock()
        defer { lock.unlock() }

        let fileCoordinator = NSFileCoordinator(filePresenter: self)

        try fileCoordinator.coordinate(writingItemAt: fileURL, options: [.forReplacing]) { fileURL in
            try JSONEncoder().encode(content).write(to: fileURL)
        }

        cachedContent = content
    }

    public func clear() throws {
        lock.lock()
        defer { lock.unlock() }

        let fileCoordinator = NSFileCoordinator(filePresenter: self)
        try fileCoordinator.coordinate(writingItemAt: fileURL, options: [.forDeleting]) { fileURL in
            try FileManager.default.removeItem(at: fileURL)
        }

        cachedContent = nil
    }

    // MARK: - NSFilePresenter

    /// Called when the file content has changed by another process.
    public func presentedItemDidChange() {
        lock.lock()
        defer { lock.unlock() }

        // Invalidate the cache so the next read() fetches fresh data from disk
        cachedContent = nil

        DispatchQueue.main.async {
            self.loadInitialCache()
        }
    }

    /// Called when the file is being deleted by another process.
    public func accommodatePresentedItemDeletion(completionHandler: @escaping ((any Error)?) -> Void) {
        lock.lock()
        cachedContent = nil
        lock.unlock()

        completionHandler(nil)
    }
}
