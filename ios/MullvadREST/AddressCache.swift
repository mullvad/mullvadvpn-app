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

        /// Memory cache.
        private var cachedAddresses: CachedAddresses = defaultCachedAddresses

        /// Cache file location.
        private let cacheFileURL: URL

        /// The location of pre-bundled address cache file.
        private let prebundledCacheFileURL: URL

        /// Lock used for synchronizing access to instance members.
        private let nslock = NSLock()

        /// Whether address cache is in readonly mode.
        private var isReadOnly: Bool

        private static let defaultCachedAddresses = CachedAddresses(
            updatedAt: Date(timeIntervalSince1970: 0),
            endpoints: [REST.defaultAPIEndpoint]
        )

        /// Designated initializer.
        public init?(securityGroupIdentifier: String, isReadOnly: Bool) {
            let cacheFilename = "api-ip-address.json"

            guard let containerURL = FileManager.default.containerURL(
                forSecurityApplicationGroupIdentifier: securityGroupIdentifier
            ), let prebundledCacheFileURL = Bundle(for: AddressCache.self).url(
                forResource: cacheFilename,
                withExtension: nil
            ) else { return nil }

            let cacheFileURL = containerURL.appendingPathComponent(
                cacheFilename,
                isDirectory: false
            )

            self.cacheFileURL = cacheFileURL
            self.prebundledCacheFileURL = prebundledCacheFileURL
            self.isReadOnly = isReadOnly

            initCache()
        }

        public func getCurrentEndpoint() -> AnyIPEndpoint {
            nslock.lock()
            defer { nslock.unlock() }
            return cachedAddresses.endpoints.first!
        }

        public func selectNextEndpoint(_ failedEndpoint: AnyIPEndpoint) -> AnyIPEndpoint {
            nslock.lock()
            defer { nslock.unlock() }

            var currentEndpoint = cachedAddresses.endpoints.first!

            guard failedEndpoint == currentEndpoint else {
                return currentEndpoint
            }

            cachedAddresses.endpoints.removeFirst()
            cachedAddresses.endpoints.append(failedEndpoint)

            if isReadOnly {
                refreshAddresses()
            }

            currentEndpoint = cachedAddresses.endpoints.first!

            logger.debug(
                "Failed to communicate using \(failedEndpoint). Next endpoint: \(currentEndpoint)"
            )

            if !isReadOnly {
                do {
                    try writeToDisk()
                } catch {
                    logger.error(
                        error: error,
                        message: "Failed to write address cache after selecting next endpoint."
                    )
                }
            }

            return currentEndpoint
        }

        public func setEndpoints(_ endpoints: [AnyIPEndpoint]) {
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

            if !isReadOnly {
                do {
                    try writeToDisk()
                } catch {
                    logger.error(
                        error: error,
                        message: "Failed to write address cache after setting new endpoints."
                    )
                }
            }
        }

        public func getLastUpdateDate() -> Date {
            nslock.lock()
            defer { nslock.unlock() }

            return cachedAddresses.updatedAt
        }

        // MARK: - Private

        private func initCache() {
            do {
                try initCacheInner()
            } catch {
                logger.debug("Initialized cache with default API endpoint.")

                cachedAddresses = Self.defaultCachedAddresses
            }
        }

        private func initCacheInner() throws {
            let readResult = try readFromCacheLocationWithFallback()

            switch readResult.source {
            case .disk:
                cachedAddresses = readResult.cachedAddresses

            case .bundle:
                var addresses = readResult.cachedAddresses
                addresses.endpoints.shuffle()
                cachedAddresses = addresses

                if !isReadOnly {
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
            }

            logger.debug(
                """
                Initialized cache from \(readResult.source) with \
                \(cachedAddresses.endpoints.count) endpoint(s).
                """
            )
        }

        private func readFromCacheLocationWithFallback() throws -> ReadResult {
            do {
                return try readFromCacheLocation()
            } catch {
                logger.error(
                    error: error,
                    message: "Failed to read address cache from disk. Fallback to pre-bundled cache."
                )

                do {
                    return try readFromBundle()
                } catch {
                    logger.error(
                        error: error,
                        message: "Failed to read address cache from bundle."
                    )

                    throw error
                }
            }
        }

        private func readFromCacheLocation() throws -> ReadResult {
            var result: Result<ReadResult, Swift.Error>?
            let fileCoordinator = NSFileCoordinator(filePresenter: nil)

            let accessor = { (fileURL: URL) in
                result = Result {
                    let data = try Data(contentsOf: fileURL)
                    let cachedAddresses = try JSONDecoder().decode(CachedAddresses.self, from: data)

                    if cachedAddresses.endpoints.isEmpty {
                        throw EmptyCacheError(source: .disk)
                    }

                    return ReadResult(cachedAddresses: cachedAddresses, source: .disk)
                }
            }

            var error: NSError?
            fileCoordinator.coordinate(
                readingItemAt: cacheFileURL,
                options: .withoutChanges,
                error: &error,
                byAccessor: accessor
            )

            if let error = error {
                result = .failure(error)
            }

            return try result!.get()
        }

        private func readFromBundle() throws -> ReadResult {
            let data = try Data(contentsOf: prebundledCacheFileURL)
            let endpoints = try JSONDecoder().decode([AnyIPEndpoint].self, from: data)

            let cachedAddresses = CachedAddresses(
                updatedAt: Date(timeIntervalSince1970: 0),
                endpoints: endpoints
            )

            if cachedAddresses.endpoints.isEmpty {
                throw EmptyCacheError(source: .bundle)
            }

            return ReadResult(cachedAddresses: cachedAddresses, source: .bundle)
        }

        private func writeToDisk() throws {
            precondition(!isReadOnly)

            var result: Result<Void, Swift.Error>?
            let fileCoordinator = NSFileCoordinator(filePresenter: nil)

            let accessor = { (fileURL: URL) in
                result = Result {
                    let data = try JSONEncoder().encode(self.cachedAddresses)
                    try data.write(to: fileURL)
                }
            }

            var error: NSError?
            fileCoordinator.coordinate(
                writingItemAt: cacheFileURL,
                options: [.forReplacing],
                error: &error,
                byAccessor: accessor
            )

            if let error = error {
                result = .failure(error)
            }

            return try result!.get()
        }

        private func refreshAddresses() {
            do {
                let readResult = try readFromCacheLocation()
                var newCachedAddresses = readResult.cachedAddresses

                guard Set(newCachedAddresses.endpoints) != Set(cachedAddresses.endpoints)
                else { return }

                // Move current endpoint to the top of the list
                let currentEndpoint = cachedAddresses.endpoints.first!
                if let index = newCachedAddresses.endpoints.firstIndex(of: currentEndpoint) {
                    newCachedAddresses.endpoints.remove(at: index)
                    newCachedAddresses.endpoints.insert(currentEndpoint, at: 0)
                }

                cachedAddresses = newCachedAddresses
            } catch {
                logger.error(error: error, message: "Failed to refresh address cache from disk.")
            }
        }
    }

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
}
