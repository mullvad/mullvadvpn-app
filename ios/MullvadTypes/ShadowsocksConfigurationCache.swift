//
//  ShadowsocksConfigurationCache.swift
//  MullvadTypes
//
//  Created by Marco Nikic on 2023-06-05.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Holds a shadowsocks configuration object backed by a caching mechanism shared across processes
public class ShadowsocksConfigurationCache: Caching {
    public typealias CacheType = ShadowsocksConfiguration

    public static var cacheFileName: String { "shadowsocks-cache.json" }
    public let cacheFileURL: URL

    private var _configuration: ShadowsocksConfiguration?
    private let cacheLock = NSLock()

    public init(cacheFolder: URL) {
        let cacheFileURL = cacheFolder.appendingPathComponent(
            Self.cacheFileName,
            isDirectory: false
        )

        self.cacheFileURL = cacheFileURL
    }

    /// The cached shadowsocks configuration object
    /// If there is no cache available, a configuration will be read from disk
    public var configuration: ShadowsocksConfiguration? {
        get {
            cacheLock.lock()
            defer { cacheLock.unlock() }

            if let _configuration {
                return _configuration
            }
            do {
                let diskCache = try readFromDisk()
                return diskCache
            } catch {
                return nil
            }
        }
        set {
            cacheLock.lock()
            defer { cacheLock.unlock() }

            _configuration = newValue
            if let _configuration {
                try? writeToDisk(_configuration)
            }
        }
    }
}
