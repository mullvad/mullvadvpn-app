//
//  RelayCache.swift
//  RelayCache
//
//  Created by pronebird on 06/09/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadTypes

public final class RelayCache: Caching {
    public typealias CacheType = CachedRelays

    /// Cache file location.
    public let cacheFileURL: URL
    public static let cacheFileName = "relays.json"

    public init(cacheFolder: URL) {
        let cacheFileURL = cacheFolder.appendingPathComponent(
            Self.cacheFileName,
            isDirectory: false
        )
        self.cacheFileURL = cacheFileURL
    }

    /// Safely read the cache file from disk using file coordinator and fallback to prebundled
    /// relays in case if the relay cache file is missing.
    public func read() throws -> CachedRelays {
        do {
            return try readFromDisk()
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
        try writeToDisk(record)
    }

    /// Read pre-bundled relays file from disk.
    private func readPrebundledRelays() throws -> CachedRelays {
        guard let prebundledRelaysFileURL = Bundle(for: Self.self)
            .url(forResource: "relays", withExtension: "json") else { throw POSIXError(.ENOENT) }
        let data = try Data(contentsOf: prebundledRelaysFileURL)
        let relays = try REST.Coding.makeJSONDecoder()
            .decode(REST.ServerRelaysResponse.self, from: data)

        return CachedRelays(
            relays: relays,
            updatedAt: Date(timeIntervalSince1970: 0)
        )
    }
}
