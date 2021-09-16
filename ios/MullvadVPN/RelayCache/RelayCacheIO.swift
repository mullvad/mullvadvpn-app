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
    static func defaultCacheFileURL(forSecurityApplicationGroupIdentifier appGroupIdentifier: String) -> URL? {
        let containerURL = FileManager.default.containerURL(forSecurityApplicationGroupIdentifier: appGroupIdentifier)

        return containerURL?.appendingPathComponent("relays.json")
    }

    /// The path to pre-bundled `relays.json` file.
    static var preBundledRelaysFileURL: URL? {
        return Bundle.main.url(forResource: "relays", withExtension: "json")
    }

    /// Safely read the cache file from disk using file coordinator.
    static func read(cacheFileURL: URL) -> Result<RelayCache.CachedRelays, RelayCache.Error> {
        var result: Result<RelayCache.CachedRelays, RelayCache.Error>?
        let fileCoordinator = NSFileCoordinator(filePresenter: nil)

        let accessor = { (fileURLForReading: URL) -> Void in
            // Decode data from disk
            result = Result { try Data(contentsOf: fileURLForReading) }
            .mapError { RelayCache.Error.readCache($0) }
            .flatMap { (data) in
                Result { try JSONDecoder().decode(RelayCache.CachedRelays.self, from: data) }
                .mapError { RelayCache.Error.decodeCache($0) }
            }
        }

        var error: NSError?
        fileCoordinator.coordinate(readingItemAt: cacheFileURL,
                                   options: [.withoutChanges],
                                   error: &error,
                                   byAccessor: accessor)

        if let error = error {
            result = .failure(.readCache(error))
        }

        return result!
    }

    /// Safely read the cache file from disk using file coordinator and fallback to prebundled relays in case if the
    /// relay cache file is missing.
    static func readWithFallback(cacheFileURL: URL, preBundledRelaysFileURL: URL) -> Result<RelayCache.CachedRelays, RelayCache.Error> {
        return Self.read(cacheFileURL: cacheFileURL)
            .flatMapError { (error) -> Result<RelayCache.CachedRelays, RelayCache.Error> in
                switch error {
                case .decodeCache, .readCache(CocoaError.fileReadNoSuchFile):
                    return RelayCache.IO.readPrebundledRelays(fileURL: preBundledRelaysFileURL)
                default:
                    return .failure(error)
                }
            }
    }

    /// Read pre-bundled relays file from disk.
    static func readPrebundledRelays(fileURL: URL) -> Result<RelayCache.CachedRelays, RelayCache.Error> {
        return Result { try Data(contentsOf: fileURL) }
        .mapError { RelayCache.Error.readPrebundledRelays($0) }
        .flatMap { (data) -> Result<RelayCache.CachedRelays, RelayCache.Error> in
            return Result { try REST.Coding.makeJSONDecoder().decode(REST.ServerRelaysResponse.self, from: data) }
            .mapError { RelayCache.Error.decodePrebundledRelays($0) }
            .map { (relays) -> RelayCache.CachedRelays in
                return RelayCache.CachedRelays(
                    relays: relays,
                    updatedAt: Date(timeIntervalSince1970: 0)
                )
            }
        }
    }

    /// Safely write the cache file on disk using file coordinator.
    static func write(cacheFileURL: URL, record: RelayCache.CachedRelays) -> Result<(), RelayCache.Error> {
        var result: Result<(), RelayCache.Error>?
        let fileCoordinator = NSFileCoordinator(filePresenter: nil)

        let accessor = { (fileURLForWriting: URL) -> Void in
            result = Result { try JSONEncoder().encode(record) }
            .mapError { RelayCache.Error.encodeCache($0) }
            .flatMap { (data) in
                Result { try data.write(to: fileURLForWriting) }
                .mapError { RelayCache.Error.writeCache($0) }
            }
        }

        var error: NSError?
        fileCoordinator.coordinate(writingItemAt: cacheFileURL,
                                   options: [.forReplacing],
                                   error: &error,
                                   byAccessor: accessor)

        if let error = error {
            result = .failure(.writeCache(error))
        }

        return result!
    }
}
