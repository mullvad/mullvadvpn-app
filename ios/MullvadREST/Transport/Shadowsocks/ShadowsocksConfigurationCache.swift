//
//  ShadowsocksConfigurationCache.swift
//  MullvadTransport
//
//  Created by Marco Nikic on 2023-06-05.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

/// Holds a shadowsocks configuration object backed by a caching mechanism shared across processes
public final class ShadowsocksConfigurationCache {
    private let configurationLock = NSLock()
    private var cachedConfiguration: ShadowsocksConfiguration?
    private let fileCache: FileCache<ShadowsocksConfiguration>

    public init(cacheDirectory: URL) {
        fileCache = FileCache(
            fileURL: cacheDirectory.appendingPathComponent("shadowsocks-cache.json", isDirectory: false)
        )
    }

    /// Returns configuration from memory cache if available, otherwise attempts to load it from disk cache before
    /// returning.
    public func read() throws -> ShadowsocksConfiguration {
        configurationLock.lock()
        defer { configurationLock.unlock() }

        if let cachedConfiguration {
            return cachedConfiguration
        } else {
            let readConfiguration = try fileCache.read()
            cachedConfiguration = readConfiguration
            return readConfiguration
        }
    }

    /// Replace memory cache with new configuration and attempt to persist it on disk.
    public func write(_ configuration: ShadowsocksConfiguration) throws {
        configurationLock.lock()
        defer { configurationLock.unlock() }

        cachedConfiguration = configuration
        try fileCache.write(configuration)
    }
}
