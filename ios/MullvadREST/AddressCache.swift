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
    public final class AddressCache {
        /// Logger.
        private let logger = Logger(label: "AddressCache")

        /// Disk cache.
        private let fileCache: any FileCacheProtocol<CachedAddresses>

        /// Memory cache.
        private var cache: CachedAddresses = defaultCachedAddresses

        /// Lock used for synchronizing access to instance members.
        private let cacheLock = NSLock()

        /// Whether address cache can be written to.
        private let canWriteToCache: Bool

        /// The default set of endpoints to use as a fallback mechanism
        private static let defaultCachedAddresses = CachedAddresses(
            updatedAt: Date(timeIntervalSince1970: 0),
            endpoints: [REST.defaultAPIEndpoint]
        )

        // MARK: - Public API

        /// Designated initializer.
        public init(canWriteToCache: Bool, cacheDirectory: URL) {
            fileCache = FileCache(
                fileURL: cacheDirectory.appendingPathComponent("api-ip-address.json", isDirectory: false)
            )
            self.canWriteToCache = canWriteToCache
        }

        /// Initializer that accepts a file cache implementation and can be used in tests.
        init(canWriteToCache: Bool, fileCache: some FileCacheProtocol<CachedAddresses>) {
            self.fileCache = fileCache
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
                    cache = try fileCache.read()
                    if let firstEndpoint = cache.endpoints.first {
                        currentEndpoint = firstEndpoint
                    }
                } catch {
                    logger.error(error: error, message: "Failed to read address cache from disk.")
                }
            }
            return currentEndpoint
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

            guard canWriteToCache else { return }
            do {
                try fileCache.write(cache)
            } catch {
                logger.error(
                    error: error,
                    message: "Failed to write address cache after setting new endpoints."
                )
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

        /// Initializes cache by reading it from file on disk.
        ///
        /// If no cache file is present, a default API endpoint will be selected instead.
        public func loadFromFile() {
            cacheLock.withLock {
                // The first time the application is ran, this statement will fail as there is no cache. This is fine.
                // The cache will be filled when either `getCurrentEndpoint()` or `setEndpoints()` are called.
                do {
                    cache = try fileCache.read()
                } catch {
                    // Log all errors except when file is not on disk.
                    if (error as? CocoaError)?.code != .fileReadNoSuchFile {
                        logger.error(error: error, message: "Failed to load address cache from disk.")
                    }

                    logger.debug("Initialized cache with default API endpoint.")
                    cache = Self.defaultCachedAddresses
                }
            }
        }
    }

    public struct CachedAddresses: Codable, Equatable {
        /// Date when the cached addresses were last updated.
        var updatedAt: Date

        /// API endpoints.
        var endpoints: [AnyIPEndpoint]
    }
}
