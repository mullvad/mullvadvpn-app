//
//  AddressCacheStore.swift
//  MullvadVPN
//
//  Created by pronebird on 08/12/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Logging

extension AddressCache {

    struct CachedAddresses: Codable {
        /// Date when the cached addresses were last updated.
        var updatedAt: Date

        /// API endpoints.
        var endpoints: [AnyIPEndpoint]
    }

    enum CacheSource: CustomStringConvertible {
        /// Cache file originates from disk location.
        case disk

        /// Cache file originates from application bundle.
        case bundle

        var description: String {
            switch self {
            case .disk:
                return "disk"
            case .bundle:
                return "bundle"
            }
        }
    }

    struct ReadResult {
        var cachedAddresses: CachedAddresses
        var source: CacheSource
    }

    class Store {

        static let shared: Store = {
            let cacheFilename = "api-ip-address.json"
            let cacheDirectoryURL = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first!
            let cacheFileURL = cacheDirectoryURL.appendingPathComponent(cacheFilename, isDirectory: false)
            let prebundledCacheFileURL = Bundle.main.url(forResource: cacheFilename, withExtension: nil)!

            return Store(
                cacheFileURL: cacheFileURL,
                prebundledCacheFileURL: prebundledCacheFileURL
            )
        }()

        static var defaultCachedAddresses: CachedAddresses {
            return CachedAddresses(
                updatedAt: Date(timeIntervalSince1970: 0),
                endpoints: [
                    ApplicationConfiguration.defaultAPIEndpoint
                ]
            )
        }

        /// Logger.
        private let logger = Logger(label: "AddressCache.Store")

        /// Memory cache.
        private var cachedAddresses: CachedAddresses

        /// Cache file location.
        private let cacheFileURL: URL

        /// The location of pre-bundled address cache file.
        private let prebundledCacheFileURL: URL

        /// Lock used for synchronizing access to instance members.
        private let nslock = NSLock()

        /// Designated initializer
        init(cacheFileURL: URL, prebundledCacheFileURL: URL) {
            self.cacheFileURL = cacheFileURL
            self.prebundledCacheFileURL = prebundledCacheFileURL
            cachedAddresses = Self.defaultCachedAddresses

            initializeStore()
        }

        func getCurrentEndpoint() -> AnyIPEndpoint {
            nslock.lock()
            defer { nslock.unlock() }
            return cachedAddresses.endpoints.first!
        }

        func selectNextEndpoint(_ failedEndpoint: AnyIPEndpoint) -> AnyIPEndpoint {
            nslock.lock()
            defer { nslock.unlock() }

            var currentEndpoint = cachedAddresses.endpoints.first!

            guard failedEndpoint == currentEndpoint else {
                return currentEndpoint
            }

            cachedAddresses.endpoints.removeFirst()
            cachedAddresses.endpoints.append(failedEndpoint)

            currentEndpoint = cachedAddresses.endpoints.first!

            logger.debug("Failed to communicate using \(failedEndpoint). Next endpoint: \(currentEndpoint)")

            do {
                try writeToDisk()
            } catch {
                logger.error(
                    chainedError: AnyChainedError(error),
                    message: "Failed to write address cache after selecting next endpoint."
                )
            }

            return currentEndpoint
        }

        func setEndpoints(_ endpoints: [AnyIPEndpoint]) throws {
            nslock.lock()
            defer { nslock.unlock() }

            guard !endpoints.isEmpty else {
                throw StoreError.emptyAddressList
            }

            if Set(cachedAddresses.endpoints) == Set(endpoints) {
                cachedAddresses.updatedAt = Date()
            } else {
                // Shuffle new endpoints
                var newEndpoints = endpoints.shuffled()

                // Move current endpoint to the top of the list
                let currentEndpoint = cachedAddresses.endpoints.first!
                if let index = newEndpoints.firstIndex(of: currentEndpoint) {
                    newEndpoints.remove(at: index)
                    newEndpoints.insert(currentEndpoint, at: 0)
                }

                cachedAddresses = CachedAddresses(
                    updatedAt: Date(),
                    endpoints: newEndpoints
                )
            }

            try writeToDisk()
        }

        func getLastUpdateDate() -> Date {
            nslock.lock()
            defer { nslock.unlock() }

            return cachedAddresses.updatedAt
        }

        private func initializeStore() {
            let readResult: ReadResult
            do {
                readResult = try readFromCacheLocationWithFallback()
            } catch {
                logger.error(
                    chainedError: AnyChainedError(error),
                    message: "Failed to read address cache. Fallback to default API endpoint."
                )

                cachedAddresses = Self.defaultCachedAddresses

                logger.debug("Initialized cache with default API endpoint.")
                return
            }

            guard !readResult.cachedAddresses.endpoints.isEmpty else {
                logger.debug("Read empty cache from \(readResult.source). Fallback to default API endpoint.")

                cachedAddresses = Self.defaultCachedAddresses

                logger.debug("Initialized cache with default API endpoint.")

                return
            }

            switch readResult.source {
            case .disk:
                cachedAddresses = readResult.cachedAddresses

            case .bundle:
                var addresses = readResult.cachedAddresses
                addresses.endpoints.shuffle()
                cachedAddresses = addresses

                logger.debug("Persist address list read from bundle.")

                do {
                    try writeToDisk()
                } catch {
                    logger.error(
                        chainedError: AnyChainedError(error),
                        message: "Failed to persist address cache after reading it from bundle."
                    )
                }
            }

            logger.debug("Initialized cache from \(readResult.source) with \(cachedAddresses.endpoints.count) endpoint(s).")
        }

        private func readFromCacheLocationWithFallback() throws -> ReadResult {
            do {
                return ReadResult(
                    cachedAddresses: try readFromCacheLocation(),
                    source: .disk
                )
            } catch {
                logger.error(
                    chainedError: AnyChainedError(error),
                    message: "Failed to read address cache from disk. Fallback to pre-bundled cache."
                )
            }

            return ReadResult(
                cachedAddresses: try readFromBundle(),
                source: .bundle
            )
        }

        private func readFromCacheLocation() throws -> CachedAddresses {
            do {
                let data = try Data(contentsOf: cacheFileURL)

                return try JSONDecoder().decode(CachedAddresses.self, from: data)
            } catch let error as DecodingError {
                throw StoreError.decodeCache(error)
            } catch {
                throw StoreError.readCache(error)
            }
        }

        private func writeToDisk() throws {
            let cacheDirectoryURL = cacheFileURL.deletingLastPathComponent()

            try? FileManager.default.createDirectory(
                at: cacheDirectoryURL,
                withIntermediateDirectories: true,
                attributes: nil
            )

            do {
                let data = try JSONEncoder().encode(cachedAddresses)
                try data.write(to: cacheFileURL, options: .atomic)
            } catch let error as EncodingError {
                throw StoreError.encodeCache(error)
            } catch {
                throw StoreError.writeCache(error)
            }
        }

        private func readFromBundle() throws -> CachedAddresses {
            do {
                let data = try Data(contentsOf: prebundledCacheFileURL)
                let endpoints = try JSONDecoder().decode([AnyIPEndpoint].self, from: data)

                return CachedAddresses(
                    updatedAt: Date(timeIntervalSince1970: 0),
                    endpoints: endpoints
                )
            } catch let error as DecodingError {
                throw StoreError.decodeCacheFromBundle(error)
            } catch {
                throw StoreError.decodeCacheFromBundle(error)
            }
        }

    }
}
