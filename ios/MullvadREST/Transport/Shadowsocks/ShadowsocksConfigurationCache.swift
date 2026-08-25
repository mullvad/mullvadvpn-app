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

/// Holds a shadowsocks configuration object backed by a caching mechanism shared across processes.
public actor ShadowsocksConfigurationCache: ShadowsocksConfigurationCacheProtocol {
    private var cachedConfiguration: ShadowsocksConfiguration?
    private nonisolated let fileCache: FileCache<ShadowsocksConfiguration>

    public init(cacheDirectory: URL) {
        fileCache = FileCache(
            fileURL: cacheDirectory.appendingPathComponent("shadowsocks-cache.json", isDirectory: false)
        )
    }

    // MARK: - Primary async interface

    /// Returns configuration from memory cache if available, otherwise loads from disk cache.
    public func read() async throws -> ShadowsocksConfiguration {
        if let cachedConfiguration {
            return cachedConfiguration
        }
        let readConfiguration = try await fileCache.read()
        cachedConfiguration = readConfiguration
        return readConfiguration
    }

    /// Replaces memory cache with new configuration and persists it to disk.
    public func write(_ configuration: ShadowsocksConfiguration) async throws {
        cachedConfiguration = configuration
        try await fileCache.write(configuration)
    }

    /// Clears cached configuration.
    public func clear() async throws {
        cachedConfiguration = nil
        try await fileCache.clear()
    }

    // MARK: - Synchronous shims
    // These block the calling thread via DispatchSemaphore. Will be removed once all call
    // sites have been migrated.

    public nonisolated func read() throws -> ShadowsocksConfiguration {
        try SynchRunner.run {
            try await self.read()
        }
    }

    public nonisolated func write(_ configuration: ShadowsocksConfiguration) throws {
        try SynchRunner.run {
            try await self.write(configuration)
        }
    }

    public nonisolated func clear() throws {
        try SynchRunner.run {
            try await self.clear()
        }
    }
}
