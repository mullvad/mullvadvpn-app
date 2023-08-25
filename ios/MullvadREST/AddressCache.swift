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
        private let fileCache: any FileCacheProtocol<StoredAddressCache>

        /// Memory cache.
        private var cache: StoredAddressCache = defaultStoredCache

        /// Lock used for synchronizing access to instance members.
        private let cacheLock = NSLock()

        /// Indicates which target requested access address cache. Main bundle has read/write access to cache, while packet tunnel can only read.
        private let appTarget: ApplicationTarget

        /// The default endpoint to use as a fallback mechanism.
        private static let defaultStoredCache = StoredAddressCache(
            updatedAt: Date(timeIntervalSince1970: 0),
            endpoint: REST.defaultAPIEndpoint
        )

        // MARK: - Public API

        /// Designated initializer.
        public init(appTarget: ApplicationTarget, cacheDirectory: URL) {
            fileCache = FileCache(
                fileURL: cacheDirectory.appendingPathComponent("api-ip-address.json", isDirectory: false)
            )
            self.appTarget = appTarget
        }

        /// Initializer that accepts a file cache implementation and can be used in tests.
        init(appTarget: ApplicationTarget, fileCache: some FileCacheProtocol<StoredAddressCache>) {
            self.fileCache = fileCache
            self.appTarget = appTarget
        }

        /// Returns the latest available endpoint
        ///
        /// When running from the Network Extension, this method will read from the cache before returning.
        /// - Returns: The latest available endpoint, or a default endpoint if no endpoints are available
        public func getCurrentEndpoint() -> AnyIPEndpoint {
            cacheLock.withLock {
                switch appTarget {
                case .mainApp:
                    return cache.endpoint

                case .packetTunnel:
                    // Reload from disk cache when in the Network Extension as there is no `AddressCacheTracker` running
                    // there
                    do {
                        cache = try fileCache.read()
                    } catch {
                        logger.error(error: error, message: "Failed to read address cache from disk.")
                    }

                    return cache.endpoint
                }
            }
        }

        /// Updates the available endpoints to use
        ///
        /// Only the first available endpoint is kept, the rest are discarded.
        /// This method will only modify the on disk cache when running from the UI process.
        /// - Parameter endpoints: The new endpoints to use for API requests
        public func setEndpoints(_ endpoints: [AnyIPEndpoint]) {
            cacheLock.withLock {
                guard let firstEndpoint = endpoints.first else { return }

                cache = StoredAddressCache(updatedAt: Date(), endpoint: firstEndpoint)

                switch appTarget {
                case .mainApp:
                    do {
                        try fileCache.write(cache)
                    } catch {
                        logger.error(
                            error: error,
                            message: "Failed to write address cache after setting new endpoints."
                        )
                    }

                case .packetTunnel:
                    break
                }
            }
        }

        /// The `Date` when the cache was last updated at
        ///
        /// - Returns: The `Date` when the cache was last updated at
        public func getLastUpdateDate() -> Date {
            return cacheLock.withLock { cache.updatedAt }
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
                    cache = Self.defaultStoredCache
                }
            }
        }
    }

    /// Serializable address cache representation stored on disk.
    struct StoredAddressCache: Codable, Equatable {
        /// Date when the cached addresses were last updated.
        var updatedAt: Date

        /// API endpoint.
        var endpoint: AnyIPEndpoint
    }
}
