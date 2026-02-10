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

/// File cache implementation that can read and write any `Codable` content and uses file coordinator to coordinate I/O.
/// Caches content in memory after initial load and invalidates the cache when file changes are detected from other processes.
///
/// - Important: Only a single `FileCache` instance should exist per file URL within a process. Multiple instances
///   backed by the same file will deadlock under concurrent writes due to `NSFileCoordinator` presenter limitations,
///   and presenter notifications between same-process instances are not reliably delivered.
///
/// File coordination handles cross-process and cross-thread serialization of disk I/O. A separate serial dispatch queue
/// protects the in-memory cache against data races with presenter callbacks, which on iOS 17 may fire even when the
/// coordinator is initialized with `filePresenter: self`.
public final class FileCache<Content: Codable>: NSObject, FileCacheProtocol, NSFilePresenter, @unchecked Sendable {
    public let fileURL: URL
    public var presentedItemURL: URL? { fileURL }
    public let presentedItemOperationQueue = OperationQueue()

    /// In-memory cache of the content, only accessed on `cacheQueue`.
    private var cachedContent: Content?

    /// Serial queue that synchronizes all access to `cachedContent`.
    private let cacheQueue = DispatchQueue(label: "net.mullvadvpn.FileCache.cacheQueue")

    public init(fileURL: URL) {
        self.fileURL = fileURL
        super.init()

        presentedItemOperationQueue.maxConcurrentOperationCount = 1

        // Register as file presenter to receive change notifications
        NSFileCoordinator.addFilePresenter(self)

        // Load initial content off the main thread
        DispatchQueue.global().async {
            _ = try? self.read()
        }
    }

    deinit {
        NSFileCoordinator.removeFilePresenter(self)
    }

    public func read() throws -> Content {
        // Fast path: return cached content without file coordination.
        if let cached = cacheQueue.sync(execute: { cachedContent }) {
            return cached
        }

        let fileCoordinator = NSFileCoordinator(filePresenter: self)

        return try fileCoordinator.coordinate(readingItemAt: fileURL, options: [.withoutChanges]) { url in
            // Re-check after acquiring coordination; another thread may have populated the cache.
            if let cached = cacheQueue.sync(execute: { cachedContent }) {
                return cached
            }

            let content = try JSONDecoder().decode(Content.self, from: Data(contentsOf: url))
            cacheQueue.sync { cachedContent = content }
            return content
        }
    }

    public func write(_ content: Content) throws {
        let fileCoordinator = NSFileCoordinator(filePresenter: self)

        try fileCoordinator.coordinate(writingItemAt: fileURL, options: [.forReplacing]) { url in
            try JSONEncoder().encode(content).write(to: url)
            cacheQueue.sync { cachedContent = content }
        }
    }

    public func clear() throws {
        let fileCoordinator = NSFileCoordinator(filePresenter: self)

        try fileCoordinator.coordinate(writingItemAt: fileURL, options: [.forDeleting]) { url in
            try FileManager.default.removeItem(at: url)
            cacheQueue.sync { cachedContent = nil }
        }
    }

    // MARK: - NSFilePresenter

    /// Called when the file content has changed by another process.
    public func presentedItemDidChange() {
        let content = try? JSONDecoder().decode(
            Content.self,
            from: Data(contentsOf: fileURL)
        )
        cacheQueue.sync { cachedContent = content }
    }

    /// Called when the file is being deleted by another process.
    public func accommodatePresentedItemDeletion(completionHandler: @escaping ((any Error)?) -> Void) {
        cacheQueue.sync { cachedContent = nil }
        completionHandler(nil)
    }
}
