//
//  RelayCache.swift
//  RelayCache
//
//  Created by pronebird on 06/09/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST

public final class RelayCache {
    /// Cache file location.
    let cacheFileURL: URL

    /// Location of pre-bundled relays file.
    let prebundledRelaysFileURL: URL

    /// Initialize cache with default cache file location in app group container.
    public init?(securityGroupIdentifier: String) {
        guard let containerURL = FileManager.default.containerURL(
            forSecurityApplicationGroupIdentifier: securityGroupIdentifier
        ), let prebundledRelaysFileURL = Bundle(for: Self.self)
            .url(forResource: "relays", withExtension: "json") else { return nil }

        cacheFileURL = containerURL.appendingPathComponent("relays.json", isDirectory: false)
        self.prebundledRelaysFileURL = prebundledRelaysFileURL
    }

    /// Safely read the cache file from disk using file coordinator and fallback to prebundled
    /// relays in case if the relay cache file is missing.
    public func read() throws -> CachedRelays {
        do {
            return try readDiskCache()
        } catch {
            if error is DecodingError || (error as? CocoaError)?.code == .fileReadNoSuchFile {
                return try readPrebundledRelays()
            } else {
                throw error
            }
        }
    }

    /// Safely write the cache file on disk using file coordinator.
    public func write(record: CachedRelays) throws {
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

    /// Safely read the cache file from disk using file coordinator.
    private func readDiskCache() throws -> CachedRelays {
        var result: Result<CachedRelays, Error>?
        let fileCoordinator = NSFileCoordinator(filePresenter: nil)

        let accessor = { (fileURLForReading: URL) in
            result = Result {
                let data = try Data(contentsOf: fileURLForReading)
                return try JSONDecoder().decode(CachedRelays.self, from: data)
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

    /// Read pre-bundled relays file from disk.
    private func readPrebundledRelays() throws -> CachedRelays {
        let data = try Data(contentsOf: prebundledRelaysFileURL)
        let relays = try REST.Coding.makeJSONDecoder()
            .decode(REST.ServerRelaysResponse.self, from: data)

        return CachedRelays(
            relays: relays,
            updatedAt: Date(timeIntervalSince1970: 0)
        )
    }
}
