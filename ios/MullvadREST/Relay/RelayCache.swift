//
//  RelayCache.swift
//  RelayCache
//
//  Created by pronebird on 06/09/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

public protocol RelayCacheProtocol: Sendable {
    /// Reads from a cached list,
    /// which falls back to reading from prebundled relays if there was no cache hit
    func read() throws -> StoredRelays
    /// Reads the relays file that were prebundled with the app installation.
    ///
    /// > Warning: Prefer `read()` over this unless there is an explicit need to read
    /// relays from the bundle, because those might contain stale data.
    func readPrebundledRelays() throws -> StoredRelays
    func write(record: StoredRelays) throws
}

/// - Warning: `RelayCache` should not be used directly. It should be used through `IPOverrideWrapper` to have
/// ip overrides applied.
public final class RelayCache: RelayCacheProtocol, Sendable {
    private let fileURL: URL
    nonisolated(unsafe) private let fileCache: any FileCacheProtocol<StoredRelays>

    /// Designated initializer
    public init(cacheDirectory: URL) {
        fileURL = cacheDirectory.appendingPathComponent("relays.json", isDirectory: false)
        fileCache = FileCache(fileURL: fileURL)
    }

    /// Initializer that accepts a custom FileCache implementation. Used in tests.
    init(fileCache: some FileCacheProtocol<StoredRelays>) {
        fileURL = FileManager.default.temporaryDirectory.appendingPathComponent("relays.json", isDirectory: false)
        self.fileCache = fileCache
    }

    /// Safely read the cache file from disk using file coordinator and fallback in the following manner:
    /// 1. If there is a file but it's not decodable, try to parse into the old cache format. If it's still
    ///    not decodable, read the pre-bundled data.
    /// 2. If there is no file, read from the pre-bundled data.
    public func read() throws -> StoredRelays {
        do {
            return try fileCache.read()
        } catch is DecodingError {
            do {
                let oldFormatFileCache = FileCache<CachedRelays>(fileURL: fileURL)
                return try StoredRelays(cachedRelays: try oldFormatFileCache.read())
            } catch {
                return try readPrebundledRelays()
            }
        } catch {
            return try readPrebundledRelays()
        }
    }

    /// Safely write the cache file on disk using file coordinator.
    public func write(record: StoredRelays) throws {
        try fileCache.write(record)
    }

    /// Read pre-bundled relays file from disk.
    public func readPrebundledRelays() throws -> StoredRelays {
        guard let prebundledRelaysFileURL = Bundle(for: Self.self).url(forResource: "relays", withExtension: "json")
        else { throw CocoaError(.fileNoSuchFile) }

        let data = try Data(contentsOf: prebundledRelaysFileURL)

        return try StoredRelays(
            rawData: data,
            updatedAt: Date(timeIntervalSince1970: 0)
        )
    }
}
