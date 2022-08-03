//
//  RelayCacheIO.swift
//  RelayCacheIO
//
//  Created by pronebird on 27/07/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension RelayCache {
    enum IO {}
}

extension RelayCache.IO {
    /// The default cache file location bound by app group container.
    static func defaultCacheFileURL(
        forSecurityApplicationGroupIdentifier appGroupIdentifier: String
    ) -> URL? {
        let containerURL = FileManager.default.containerURL(
            forSecurityApplicationGroupIdentifier: appGroupIdentifier
        )

        return containerURL?.appendingPathComponent("relays.json")
    }

    /// The path to pre-bundled `relays.json` file.
    static var preBundledRelaysFileURL: URL? {
        return Bundle.main.url(forResource: "relays", withExtension: "json")
    }

    /// Safely read the cache file from disk using file coordinator.
    static func read(cacheFileURL: URL) throws -> RelayCache.CachedRelays {
        var result: Result<RelayCache.CachedRelays, Error>?
        let fileCoordinator = NSFileCoordinator(filePresenter: nil)

        let accessor = { (fileURLForReading: URL) in
            result = Result {
                let data = try Data(contentsOf: fileURLForReading)
                return try JSONDecoder().decode(RelayCache.CachedRelays.self, from: data)
            }
        }

        var error: NSError?
        fileCoordinator.coordinate(
            readingItemAt: cacheFileURL,
            options: [.withoutChanges],
            error: &error,
            byAccessor: accessor
        )

        if let error = error {
            result = .failure(error)
        }

        return try result!.get()
    }

    /// Safely read the cache file from disk using file coordinator and fallback to prebundled
    /// relays in case if the relay cache file is missing.
    static func readWithFallback(cacheFileURL: URL, preBundledRelaysFileURL: URL)
        throws -> RelayCache.CachedRelays
    {
        do {
            return try Self.read(cacheFileURL: cacheFileURL)
        } catch {
            if error is DecodingError || (error as? CocoaError)?.code == .fileReadNoSuchFile {
                return try Self.readPrebundledRelays(fileURL: preBundledRelaysFileURL)
            } else {
                throw error
            }
        }
    }

    /// Read pre-bundled relays file from disk.
    static func readPrebundledRelays(fileURL: URL) throws -> RelayCache.CachedRelays {
        let data = try Data(contentsOf: fileURL)
        let relays = try REST.Coding.makeJSONDecoder()
            .decode(REST.ServerRelaysResponse.self, from: data)

        return RelayCache.CachedRelays(
            relays: relays,
            updatedAt: Date(timeIntervalSince1970: 0)
        )
    }

    /// Safely write the cache file on disk using file coordinator.
    static func write(cacheFileURL: URL, record: RelayCache.CachedRelays) throws {
        var result: Result<Void, Error>?
        let fileCoordinator = NSFileCoordinator(filePresenter: nil)

        let accessor = { (fileURLForWriting: URL) in
            result = Result {
                let data = try JSONEncoder().encode(record)
                try data.write(to: fileURLForWriting)
            }
        }

        var error: NSError?
        fileCoordinator.coordinate(
            writingItemAt: cacheFileURL,
            options: [.forReplacing],
            error: &error,
            byAccessor: accessor
        )

        if let error = error {
            result = .failure(error)
        }

        try result?.get()
    }
}
