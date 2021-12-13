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

        /// Queue used for synchronizing access to instance members.
        private let stateQueue = DispatchQueue(label: "AddressCacheStoreQueue")

        /// Designated initializer
        init(cacheFileURL: URL, prebundledCacheFileURL: URL) {
            self.cacheFileURL = cacheFileURL
            self.prebundledCacheFileURL = prebundledCacheFileURL
            self.cachedAddresses = Self.defaultCachedAddresses

            switch readFromCacheLocationWithFallback() {
            case .success(let readResult):
                if readResult.cachedAddresses.endpoints.isEmpty {
                    logger.debug("Read empty cache from \(readResult.source). Fallback to default API endpoint.")

                    cachedAddresses = Self.defaultCachedAddresses

                    logger.debug("Initialized cache with default API endpoint.")
                } else {
                    switch readResult.source {
                    case .disk:
                        cachedAddresses = readResult.cachedAddresses

                    case .bundle:
                        var addresses = readResult.cachedAddresses
                        addresses.endpoints.shuffle()
                        cachedAddresses = addresses

                        logger.debug("Persist address list read from bundle.")

                        if case .failure(let error) = self.writeToDisk() {
                            logger.error(chainedError: error, message: "Failed to persist address cache after reading it from bundle.")
                        }
                    }

                    logger.debug("Initialized cache from \(readResult.source) with \(cachedAddresses.endpoints.count) endpoint(s).")
                }

            case .failure(let error):
                logger.error(chainedError: error, message: "Failed to read address cache. Fallback to default API endpoint.")

                cachedAddresses = Self.defaultCachedAddresses

                logger.debug("Initialized cache with default API endpoint.")
            }
        }

        func getCurrentEndpoint(_ completionHandler: @escaping (AnyIPEndpoint) -> Void) {
            stateQueue.async {
                let currentEndpoint = self.cachedAddresses.endpoints.first!

                completionHandler(currentEndpoint)
            }
        }

        func selectNextEndpoint(_ failedEndpoint: AnyIPEndpoint, completionHandler: @escaping (AnyIPEndpoint) -> Void) {
            stateQueue.async {
                var currentEndpoint = self.cachedAddresses.endpoints.first!

                if failedEndpoint == currentEndpoint {
                    self.cachedAddresses.endpoints.removeFirst()
                    self.cachedAddresses.endpoints.append(failedEndpoint)

                    currentEndpoint = self.cachedAddresses.endpoints.first!

                    self.logger.debug("Failed to communicate using \(failedEndpoint). Next endpoint: \(currentEndpoint)")

                    if case .failure(let error) = self.writeToDisk() {
                        self.logger.error(chainedError: error, message: "Failed to write address cache after selecting next endpoint.")
                    }
                }

                completionHandler(currentEndpoint)
            }
        }

        func setEndpoints(_ endpoints: [AnyIPEndpoint], completionHandler: @escaping (AddressCache.StoreError?) -> Void) {
            stateQueue.async {
                guard !endpoints.isEmpty else {
                    completionHandler(.emptyAddressList)
                    return
                }

                if Set(self.cachedAddresses.endpoints) == Set(endpoints) {
                    self.cachedAddresses.updatedAt = Date()
                } else {
                    // Shuffle new endpoints
                    var newEndpoints = endpoints.shuffled()

                    // Move current endpoint to the top of the list
                    let currentEndpoint = self.cachedAddresses.endpoints.first!
                    if let index = newEndpoints.firstIndex(of: currentEndpoint) {
                        newEndpoints.remove(at: index)
                        newEndpoints.insert(currentEndpoint, at: 0)
                    }

                    self.cachedAddresses = CachedAddresses(
                        updatedAt: Date(),
                        endpoints: newEndpoints
                    )
                }

                let writeResult = self.writeToDisk()

                completionHandler(writeResult.error)
            }
        }

        func getLastUpdateDate(_ completionHandler: @escaping (Date) -> Void) {
            stateQueue.async {
                completionHandler(self.cachedAddresses.updatedAt)
            }
        }

        func getLastUpdateDateAndWait() -> Date {
            return stateQueue.sync {
                return self.cachedAddresses.updatedAt
            }
        }

        private func readFromCacheLocationWithFallback() -> Result<ReadResult, AddressCache.StoreError> {
            return readFromCacheLocation()
                .map { addresses in
                    return ReadResult(
                        cachedAddresses: addresses,
                        source: .disk
                    )
                }
                .flatMapError { error in
                    logger.error(chainedError: error, message: "Failed to read address cache from disk. Fallback to pre-bundled cache.")

                    return readFromBundle().map { cachedAddresses in
                        return ReadResult(
                            cachedAddresses: cachedAddresses,
                            source: .bundle
                        )
                    }
                }
        }

        private func readFromCacheLocation() -> Result<CachedAddresses, AddressCache.StoreError> {
            return Result { try Data(contentsOf: cacheFileURL) }
                .mapError { error in
                    return .readCache(error)
                }
                .flatMap { data in
                    return Result { try JSONDecoder().decode(CachedAddresses.self, from: data) }
                        .mapError { error in
                            return .decodeCache(error)
                        }
                }
        }

        private func writeToDisk() -> Result<(), AddressCache.StoreError> {
            let cacheDirectoryURL = cacheFileURL.deletingLastPathComponent()

            try? FileManager.default.createDirectory(
                at: cacheDirectoryURL,
                withIntermediateDirectories: true,
                attributes: nil
            )

            return Result { try JSONEncoder().encode(cachedAddresses) }
                .mapError { error in
                    return .encodeCache(error)
                }
                .flatMap { data in
                    return Result { try data.write(to: cacheFileURL, options: .atomic) }
                        .mapError { error in
                            return .writeCache(error)
                        }
                }
        }

        private func readFromBundle() -> Result<CachedAddresses, AddressCache.StoreError> {
            return Result { try Data(contentsOf: prebundledCacheFileURL) }
                .mapError { error in
                    return .readCacheFromBundle(error)
                }
                .flatMap { data in
                    return Result { try JSONDecoder().decode([AnyIPEndpoint].self, from: data) }
                        .mapError { error in
                            return .decodeCacheFromBundle(error)
                        }
                        .map { endpoints in
                            return CachedAddresses(
                                updatedAt: Date(timeIntervalSince1970: 0),
                                endpoints: endpoints
                            )
                        }
                }
        }

    }
}
