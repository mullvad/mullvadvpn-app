//
//  AddressCacheStore.swift
//  MullvadVPN
//
//  Created by pronebird on 08/12/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging

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

    struct EmptyCacheError: LocalizedError {
        let source: CacheSource

        var errorDescription: String? {
            return "Address cache file from \(source) does not contain any API addresses."
        }
    }

    class Store {
        static let shared: Store = {
            let cacheFilename = "api-ip-address.json"
            let cacheDirectoryURL = FileManager.default.urls(
                for: .applicationSupportDirectory,
                in: .userDomainMask
            ).first!
            let cacheFileURL = cacheDirectoryURL.appendingPathComponent(
                cacheFilename,
                isDirectory: false
            )
            let prebundledCacheFileURL = Bundle.main.url(
                forResource: cacheFilename,
                withExtension: nil
            )!

            return Store(
                cacheFileURL: cacheFileURL,
                prebundledCacheFileURL: prebundledCacheFileURL
            )
        }()

        static var defaultCachedAddresses: CachedAddresses {
            return CachedAddresses(
                updatedAt: Date(timeIntervalSince1970: 0),
                endpoints: [
                    ApplicationConfiguration.defaultAPIEndpoint,
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

            do {
                let readResult = try Self.readFromCacheLocationWithFallback(
                    cacheFileURL: cacheFileURL,
                    prebundledCacheFileURL: prebundledCacheFileURL,
                    logger: logger
                )

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
                            error: error,
                            message: "Failed to persist address cache after reading it from bundle."
                        )
                    }
                }

                logger.debug(
                    """
                    Initialized cache from \(readResult.source) with \
                    \(cachedAddresses.endpoints.count) endpoint(s).
                    """
                )
            } catch {
                logger.debug("Initialized cache with default API endpoint.")

                cachedAddresses = Self.defaultCachedAddresses
            }
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

            logger
                .debug(
                    "Failed to communicate using \(failedEndpoint). Next endpoint: \(currentEndpoint)"
                )

            do {
                try writeToDisk()
            } catch {
                logger.error(
                    error: error,
                    message: "Failed to write address cache after selecting next endpoint."
                )
            }

            return currentEndpoint
        }

        func setEndpoints(_ endpoints: [AnyIPEndpoint]) {
            nslock.lock()
            defer { nslock.unlock() }

            guard !endpoints.isEmpty else {
                return
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

            do {
                try writeToDisk()
            } catch {
                logger.error(
                    error: error,
                    message: "Failed to write address cache after setting new endpoints."
                )
            }
        }

        func getLastUpdateDate() -> Date {
            nslock.lock()
            defer { nslock.unlock() }

            return cachedAddresses.updatedAt
        }

        private static func readFromCacheLocationWithFallback(
            cacheFileURL: URL,
            prebundledCacheFileURL: URL,
            logger: Logger
        ) throws -> ReadResult {
            do {
                let readResult = ReadResult(
                    cachedAddresses: try readFromCacheLocation(cacheFileURL),
                    source: .disk
                )

                try checkReadResultContainsEndpoints(readResult)

                return readResult
            } catch {
                logger.error(
                    error: error,
                    message: "Failed to read address cache from disk. Fallback to pre-bundled cache."
                )

                do {
                    let readResult = ReadResult(
                        cachedAddresses: try readFromBundle(prebundledCacheFileURL),
                        source: .bundle
                    )

                    try checkReadResultContainsEndpoints(readResult)

                    return readResult
                } catch {
                    logger.error(
                        error: error,
                        message: "Failed to read address cache from bundle."
                    )

                    throw error
                }
            }
        }

        private static func checkReadResultContainsEndpoints(_ readResult: ReadResult) throws {
            if readResult.cachedAddresses.endpoints.isEmpty {
                throw EmptyCacheError(source: readResult.source)
            }
        }

        private static func readFromCacheLocation(_ cacheFileURL: URL) throws -> CachedAddresses {
            let data = try Data(contentsOf: cacheFileURL)

            return try JSONDecoder().decode(CachedAddresses.self, from: data)
        }

        private static func readFromBundle(_ prebundledCacheFileURL: URL) throws
            -> CachedAddresses
        {
            let data = try Data(contentsOf: prebundledCacheFileURL)
            let endpoints = try JSONDecoder().decode([AnyIPEndpoint].self, from: data)

            return CachedAddresses(
                updatedAt: Date(timeIntervalSince1970: 0),
                endpoints: endpoints
            )
        }

        private func writeToDisk() throws {
            let cacheDirectoryURL = cacheFileURL.deletingLastPathComponent()

            try? FileManager.default.createDirectory(
                at: cacheDirectoryURL,
                withIntermediateDirectories: true,
                attributes: nil
            )

            let data = try JSONEncoder().encode(cachedAddresses)
            try data.write(to: cacheFileURL, options: .atomic)
        }
    }
}
