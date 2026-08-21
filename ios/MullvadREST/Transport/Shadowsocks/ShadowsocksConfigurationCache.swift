// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation
import MullvadTypes

public protocol ShadowsocksConfigurationCacheProtocol: Sendable {
    func read() throws -> ShadowsocksConfiguration
    func write(_ configuration: ShadowsocksConfiguration) throws
    func clear() throws
}

/// Holds a shadowsocks configuration object backed by a caching mechanism shared across processes
public final class ShadowsocksConfigurationCache: ShadowsocksConfigurationCacheProtocol, @unchecked Sendable {
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
        try configurationLock.withLock {
            if let cachedConfiguration {
                return cachedConfiguration
            } else {
                let readConfiguration = try fileCache.read()
                cachedConfiguration = readConfiguration
                return readConfiguration
            }
        }
    }

    /// Replace memory cache with new configuration and attempt to persist it on disk.
    public func write(_ configuration: ShadowsocksConfiguration) throws {
        try configurationLock.withLock {
            cachedConfiguration = configuration
            try fileCache.write(configuration)
        }
    }

    /// Clear cached configuration.
    public func clear() throws {
        try configurationLock.withLock {
            cachedConfiguration = nil
            try fileCache.clear()
        }
    }
}
