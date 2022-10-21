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
        static var defaultCachedAddresses: CachedAddresses {
            return CachedAddresses(
                updatedAt: Date(timeIntervalSince1970: 0),
                endpoints: [
                    REST.defaultAPIEndpoint,
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

        private let accessLevel: AccessPermission

        /// Designated initializer
        public init(
            cacheFileURL: URL,
            prebundledCacheFileURL: URL,
            accessLevel: AccessPermission
        ) {
            self.cacheFileURL = cacheFileURL
            self.prebundledCacheFileURL = prebundledCacheFileURL
            self.accessLevel = accessLevel

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

        public convenience init(accessLevel: AccessPermission) {
            let cacheFilename = "api-ip-address.json"
            let cacheDirectoryURL = Self.defaultCacheFileURL(forSecurityApplicationGroupIdentifier: "group.net.mullvad.MullvadVPN")!
            
            let cacheFileURL = cacheDirectoryURL.appendingPathComponent(
                cacheFilename,
                isDirectory: false
            )

            let prebundledCacheFileURL = Bundle.main.url(
                forResource: cacheFilename,
                withExtension: nil
            )!

            self.init(
                cacheFileURL: cacheFileURL,
                prebundledCacheFileURL: prebundledCacheFileURL,
                accessLevel: accessLevel
            )
        }

        static func defaultCacheFileURL(
            forSecurityApplicationGroupIdentifier appGroupIdentifier: String
        ) -> URL? {
            let containerURL = FileManager.default.containerURL(
                forSecurityApplicationGroupIdentifier: appGroupIdentifier
            )

            return containerURL
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

            guard accessLevel.canWrite else {
                logger
                    .debug(
                        "Failed to communicate using \(failedEndpoint). Next endpoint: \(currentEndpoint)"
                    )

                var endpoints = cachedAddresses.endpoints
                endpoints.removeFirst()
                endpoints.append(failedEndpoint)
                return endpoints.first.unsafelyUnwrapped
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

        public func setEndpoints(_ endpoints: [AnyIPEndpoint]) throws {
            guard accessLevel.canWrite else { throw NotAllowedToWriteError() }

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

        public func getLastUpdateDate() -> Date {
            nslock.lock()
            defer { nslock.unlock() }

            return cachedAddresses.updatedAt
        }

        // MARK: - Private

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
            var result: Result<CachedAddresses, Swift.Error>?
            let fileCoordinator = NSFileCoordinator(filePresenter: nil)

            let accessor = { (fileURLForReading: URL) in
                result = Result {
                    let data = try Data(contentsOf: fileURLForReading)
                    return try JSONDecoder().decode(CachedAddresses.self, from: data)
                }
            }

            var error: NSError?
            fileCoordinator.coordinate(
                readingItemAt: cacheFileURL,
                options: [.withoutChanges],
                error: &error,
                byAccessor: accessor
            )

            if let error = error {
                result = .failure(error)
            }

            return try result!.get()
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

            var result: Result<Void, Swift.Error>?
            let fileCoordinator = NSFileCoordinator(filePresenter: nil)

            let accessor = {  [cachedAddresses] (fileURLForWriting: URL) in
                result = Result {
                    let data = try JSONEncoder().encode(cachedAddresses)
                    try data.write(to: fileURLForWriting)
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

            try result?.get()
        }
    }

    public enum AccessPermission {
        case readOnly
        case readWrite

        var canWrite: Bool {
            self == .readWrite
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

    public struct NotAllowedToWriteError: LocalizedError {
        public var errorDescription: String? {
            return "Writing access level is required to be able to change"
        }
    }
}
