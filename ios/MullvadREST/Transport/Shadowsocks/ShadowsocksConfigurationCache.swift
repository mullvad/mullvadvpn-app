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
///
/// Like `FileCache`, this actor runs on a dedicated `DispatchSerialQueue` so that the synchronous
/// shims can block on `queue.sync` without ever occupying a cooperative thread.
public actor ShadowsocksConfigurationCache: ShadowsocksConfigurationCacheProtocol {
    /// Only accessed while running on `queue`, mirroring `FileCache`.
    private final class Storage: @unchecked Sendable {
        var cached: ShadowsocksConfiguration?
    }

    public nonisolated var unownedExecutor: UnownedSerialExecutor {
        queue.asUnownedSerialExecutor()
    }

    private nonisolated let storage = Storage()
    private nonisolated let queue = DispatchSerialQueue(label: "net.mullvad.ShadowsocksConfigurationCache")
    private nonisolated let fileCache: FileCache<ShadowsocksConfiguration>

    public init(cacheDirectory: URL) {
        fileCache = FileCache(
            fileURL: cacheDirectory.appendingPathComponent("shadowsocks-cache.json", isDirectory: false)
        )
    }

    // MARK: - Asynchronous functions

    public func read() async throws -> ShadowsocksConfiguration {
        if let cached = storage.cached {
            return cached
        }
        let configuration = try await fileCache.read()
        storage.cached = configuration
        return configuration
    }

    public func write(_ configuration: ShadowsocksConfiguration) async throws {
        storage.cached = configuration
        try await fileCache.write(configuration)
    }

    public func clear() async throws {
        storage.cached = nil
        try await fileCache.clear()
    }

    // MARK: - Synchronous shims
    // Will be removed once all call sites have been migrated to async/await.

    public nonisolated func read() throws -> ShadowsocksConfiguration {
        try queue.sync {
            if let cached = storage.cached {
                return cached
            }
            let configuration = try fileCache.read()
            storage.cached = configuration
            return configuration
        }
    }

    public nonisolated func write(_ configuration: ShadowsocksConfiguration) throws {
        try queue.sync {
            storage.cached = configuration
            try fileCache.write(configuration)
        }
    }

    public nonisolated func clear() throws {
        try queue.sync {
            storage.cached = nil
            try fileCache.clear()
        }
    }
}
