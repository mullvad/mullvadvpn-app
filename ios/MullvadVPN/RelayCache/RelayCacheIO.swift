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
    static func read(cacheFileURL: URL) throws -> RelayCache.CachedRelays {
        var result: Result<RelayCache.CachedRelays, RelayCache.Error>?
        let fileCoordinator = NSFileCoordinator(filePresenter: nil)

        let accessor = { (fileURLForReading: URL) -> Void in
            // Decode data from disk
            do {
                let data = try Data(contentsOf: fileURLForReading)
                let relays = try JSONDecoder().decode(RelayCache.CachedRelays.self, from: data)

                result = .success(relays)
            } catch let error as DecodingError {
                result = .failure(.decodeCache(error))
            } catch {
                result = .failure(.readCache(error))
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

        return try result!.get()
    }

    /// Safely read the cache file from disk using file coordinator and fallback to prebundled relays in case if the
    /// relay cache file is missing.
    static func readWithFallback(cacheFileURL: URL, preBundledRelaysFileURL: URL) throws -> RelayCache.CachedRelays {
        do {
            return try Self.read(cacheFileURL: cacheFileURL)
        } catch {
            let error = error as! RelayCache.Error

            switch error {
            case .decodeCache, .readCache(CocoaError.fileReadNoSuchFile):
                return try RelayCache.IO.readPrebundledRelays(fileURL: preBundledRelaysFileURL)
            default:
                throw error
            }
        }
    }

    /// Read pre-bundled relays file from disk.
    static func readPrebundledRelays(fileURL: URL) throws -> RelayCache.CachedRelays {
        do {
            let data = try Data(contentsOf: fileURL)
            let relays = try REST.Coding.makeJSONDecoder()
                .decode(REST.ServerRelaysResponse.self, from: data)

            return RelayCache.CachedRelays(
                relays: relays,
                updatedAt: Date(timeIntervalSince1970: 0)
            )
        } catch let error as DecodingError {
            throw RelayCache.Error.decodePrebundledRelays(error)
        } catch {
            throw RelayCache.Error.readPrebundledRelays(error)
        }
    }

    /// Safely write the cache file on disk using file coordinator.
    static func write(cacheFileURL: URL, record: RelayCache.CachedRelays) throws {
        var resultError: RelayCache.Error?
        let fileCoordinator = NSFileCoordinator(filePresenter: nil)

        let accessor = { (fileURLForWriting: URL) -> Void in
            do {
                let data = try JSONEncoder().encode(record)
                try data.write(to: fileURLForWriting)
            } catch let error as EncodingError {
                resultError = .encodeCache(error)
            } catch {
                resultError = .writeCache(error)
            }
        }

        var error: NSError?
        fileCoordinator.coordinate(writingItemAt: cacheFileURL,
                                   options: [.forReplacing],
                                   error: &error,
                                   byAccessor: accessor)

        if let resultError = resultError {
            throw resultError
        }
    }
}
