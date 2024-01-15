//
//  RelayCache.swift
//  RelayCache
//
//  Created by pronebird on 06/09/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

public protocol RelayCacheProtocol {
    func read() throws -> CachedRelays
    func write(record: CachedRelays) throws
}

/// - Warning: `RelayCache` should not be used directly. It should be used through `IPOverrideWrapper` to have
/// ip overrides applied.
public final class RelayCache: RelayCacheProtocol {
    private let fileCache: any FileCacheProtocol<CachedRelays>

    /// Designated initializer
    public init(cacheDirectory: URL) {
        fileCache = FileCache(fileURL: cacheDirectory.appendingPathComponent("relays.json", isDirectory: false))
    }

    /// Initializer that accepts a custom FileCache implementation. Used in tests.
    init(fileCache: some FileCacheProtocol<CachedRelays>) {
        self.fileCache = fileCache
    }

    /// Safely read the cache file from disk using file coordinator and fallback to prebundled
    /// relays in case if the relay cache file is missing.
    public func read() throws -> CachedRelays {
        do {
            return try fileCache.read()
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
        try fileCache.write(record)
    }

    /// Read pre-bundled relays file from disk.
    private func readPrebundledRelays() throws -> CachedRelays {
        guard let prebundledRelaysFileURL = Bundle(for: Self.self).url(forResource: "relays", withExtension: "json")
        else { throw CocoaError(.fileNoSuchFile) }

        let data = try Data(contentsOf: prebundledRelaysFileURL)
        let relays = try REST.Coding.makeJSONDecoder().decode(REST.ServerRelaysResponse.self, from: data)

        return CachedRelays(
            relays: relays,
            updatedAt: Date(timeIntervalSince1970: 0)
        )
    }
}
