//
//  AddressCache.swift
//  MullvadREST
//
//  Created by pronebird on 08/12/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadTypes

extension REST {
    public struct CachedAddresses: Codable {
        /// Date when the cached addresses were last updated.
        var updatedAt: Date

        /// API endpoints.
        var endpoints: [AnyIPEndpoint]
    }

    public final class AddressCache: Caching {
        public typealias CacheType = CachedAddresses
        /// Logger.
        private let logger = Logger(label: "AddressCache")

        /// Memory cache.
        var cache: CachedAddresses = defaultCachedAddresses

        /// Cache file location.
        public let cacheFileURL: URL

        /// Lock used for synchronizing access to instance members.
        private let cacheLock = NSLock()

        /// Whether address cache can be written to.
        private let canWriteToCache: Bool

        /// The name of the cache file on disk
        public static let cacheFileName = "api-ip-address.json"

        /// The default set of endpoints to use as a fallback mechanism
        private static let defaultCachedAddresses = CachedAddresses(
            updatedAt: Date(timeIntervalSince1970: 0),
            endpoints: [REST.defaultAPIEndpoint]
        )

        // MARK: -

        // MARK: Public API

        /// Designated initializer.
        public init(canWriteToCache: Bool, cacheFolder: URL) {
            let cacheFileURL = cacheFolder.appendingPathComponent(
                Self.cacheFileName,
                isDirectory: false
            )

            self.cacheFileURL = cacheFileURL
            self.canWriteToCache = canWriteToCache
        }

        /// Returns the latest available endpoint
        ///
        /// When running from the Network Extension, this method will read from the cache before returning.
        /// - Returns: The latest available endpoint, or a default endpoint if no endpoints are available
        public func getCurrentEndpoint() -> AnyIPEndpoint {
            cacheLock.lock()
            defer { cacheLock.unlock() }
            var currentEndpoint = cache.endpoints.first ?? REST.defaultAPIEndpoint

            // Reload from disk cache when in the Network Extension as there is no `AddressCacheTracker` running
            // there
            if canWriteToCache == false {
                do {
                    cache = try readFromDisk()
                    if let firstEndpoint = cache.endpoints.first {
                        currentEndpoint = firstEndpoint
                    }
                } catch {
                    logger.error(error: error)
                }
            }
            return currentEndpoint
        }

        public func selectNextEndpoint(_ failedEndpoint: AnyIPEndpoint) -> AnyIPEndpoint {
            // This function currently acts as a convoluted no-op. It will be soon deleted.
            return getCurrentEndpoint()
        }

        /// Updates the available endpoints to use
        ///
        /// Only the first available endpoint is kept, the rest are discarded.
        /// This method will only modify the on disk cache when running from the UI process.
        /// - Parameter endpoints: The new endpoints to use for API requests
        public func setEndpoints(_ endpoints: [AnyIPEndpoint]) {
            cacheLock.lock()
            defer { cacheLock.unlock() }

            guard let firstEndpoint = endpoints.first else { return }
            if Set(cache.endpoints) == Set(endpoints) {
                cache.updatedAt = Date()
            } else {
                cache = CachedAddresses(
                    updatedAt: Date(),
                    endpoints: [firstEndpoint]
                )
            }

            if canWriteToCache {
                do {
                    try writeToDisk(cache)
                } catch {
                    logger.error(
                        error: error,
                        message: "Failed to write address cache after setting new endpoints."
                    )
                }
            }
        }

        /// The `Date` when the cache was last updated at
        ///
        /// - Returns: The `Date` when the cache was last updated at
        public func getLastUpdateDate() -> Date {
            cacheLock.lock()
            defer { cacheLock.unlock() }

            return cache.updatedAt
        }

        /// Initializes the cache by reading the a cached file from disk
        ///
        /// If no cache file is present, a default API endpoint will be selected instead
        public func initCache() {
            // The first time the application is ran, this statement will fail as there is no cache. This is fine.
            // The cache will be filled when either `getCurrentEndpoint` or `setEndpoints()` are called.
            do {
                cache = try readFromDisk()
            } catch {
                logger.debug("Initialized cache with default API endpoint.")
                cache = Self.defaultCachedAddresses
            }
        }
    }
}
