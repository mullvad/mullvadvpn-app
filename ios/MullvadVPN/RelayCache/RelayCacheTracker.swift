//
//  RelayCacheTracker.swift
//  MullvadVPN
//
//  Created by pronebird on 05/06/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Logging
import UIKit

extension RelayCache {
    /// Type describing the result of an attempt to fetch the new relay list from server.
    enum FetchResult: CustomStringConvertible {
        /// Request to update relays was throttled.
        case throttled

        /// Refreshed relays but the same content was found on remote.
        case sameContent

        /// Refreshed relays with new content.
        case newContent

        var description: String {
            switch self {
            case .throttled:
                return "throttled"
            case .sameContent:
                return "same content"
            case .newContent:
                return "new content"
            }
        }
    }

    struct NoCachedRelaysError: LocalizedError {
        var errorDescription: String? {
            return "Relay cache is empty."
        }
    }

    class Tracker {
        /// Relay update interval (in seconds).
        static let relayUpdateInterval: TimeInterval = 60 * 60

        /// Tracker log.
        private let logger = Logger(label: "RelayCacheTracker")

        /// The cache location used by the class instance.
        private let cacheFileURL: URL

        /// The location of prebundled `relays.json`.
        private let prebundledRelaysFileURL: URL

        /// Lock used for synchronization.
        private let nslock = NSLock()

        /// Internal operation queue.
        private let operationQueue: OperationQueue = {
            let operationQueue = AsyncOperationQueue()
            operationQueue.maxConcurrentOperationCount = 1
            return operationQueue
        }()

        /// A timer source used for periodic updates.
        private var timerSource: DispatchSourceTimer?

        /// A flag that indicates whether periodic updates are running.
        private var isPeriodicUpdatesEnabled = false

        /// API proxy.
        private let apiProxy = REST.ProxyFactory.shared.createAPIProxy()

        /// Observers.
        private let observerList = ObserverList<RelayCacheObserver>()

        /// Memory cache.
        private var cachedRelays: CachedRelays?

        /// A shared instance of `RelayCache`
        static let shared: RelayCache.Tracker = {
            let cacheFileURL = RelayCache.IO
                .defaultCacheFileURL(
                    forSecurityApplicationGroupIdentifier: ApplicationConfiguration
                        .securityGroupIdentifier
                )!
            let prebundledRelaysFileURL = RelayCache.IO.preBundledRelaysFileURL!

            return Tracker(
                cacheFileURL: cacheFileURL,
                prebundledRelaysFileURL: prebundledRelaysFileURL
            )
        }()

        private init(cacheFileURL: URL, prebundledRelaysFileURL: URL) {
            self.cacheFileURL = cacheFileURL
            self.prebundledRelaysFileURL = prebundledRelaysFileURL

            do {
                cachedRelays = try RelayCache.IO.readWithFallback(
                    cacheFileURL: cacheFileURL,
                    preBundledRelaysFileURL: prebundledRelaysFileURL
                )
            } catch {
                logger.error(
                    error: error,
                    message: "Failed to read the relay cache during initialization."
                )

                _ = updateRelays(completionHandler: nil)
            }
        }

        func startPeriodicUpdates() {
            nslock.lock()
            defer { nslock.unlock() }

            guard !isPeriodicUpdatesEnabled else { return }

            logger.debug("Start periodic relay updates.")

            isPeriodicUpdatesEnabled = true

            let nextUpdate = _getNextUpdateDate()

            scheduleRepeatingTimer(startTime: .now() + nextUpdate.timeIntervalSinceNow)
        }

        func stopPeriodicUpdates() {
            nslock.lock()
            defer { nslock.unlock() }

            guard isPeriodicUpdatesEnabled else { return }

            logger.debug("Stop periodic relay updates.")

            isPeriodicUpdatesEnabled = false

            timerSource?.cancel()
            timerSource = nil
        }

        func updateRelays(
            completionHandler: (
                (OperationCompletion<RelayCache.FetchResult, Error>) -> Void
            )? = nil
        ) -> Cancellable {
            let operation = ResultBlockOperation<RelayCache.FetchResult, Error>(
                dispatchQueue: nil
            ) { operation in
                let cachedRelays = try? self.getCachedRelays()

                if self.getNextUpdateDate() > Date() {
                    operation.finish(completion: .success(.throttled))
                    return
                }

                let task = self.apiProxy.getRelays(
                    etag: cachedRelays?.etag,
                    retryStrategy: .noRetry
                ) { completion in
                    operation.finish(
                        completion: self.handleResponse(completion: completion)
                    )
                }

                operation.addCancellationBlock {
                    task.cancel()
                }
            }

            operation.addObserver(
                BackgroundObserver(name: "Update relays", cancelUponExpiration: true)
            )

            operation.completionQueue = .main
            operation.completionHandler = completionHandler

            operationQueue.addOperation(operation)

            return operation
        }

        func getCachedRelays() throws -> CachedRelays {
            nslock.lock()
            defer { nslock.unlock() }

            if let cachedRelays = cachedRelays {
                return cachedRelays
            } else {
                throw NoCachedRelaysError()
            }
        }

        func getNextUpdateDate() -> Date {
            nslock.lock()
            defer { nslock.unlock() }

            return _getNextUpdateDate()
        }

        // MARK: - Observation

        func addObserver(_ observer: RelayCacheObserver) {
            observerList.append(observer)
        }

        func removeObserver(_ observer: RelayCacheObserver) {
            observerList.remove(observer)
        }

        // MARK: - Private

        private func _getNextUpdateDate() -> Date {
            let now = Date()

            guard let cachedRelays = cachedRelays else {
                return now
            }

            let nextUpdate = cachedRelays.updatedAt.addingTimeInterval(Self.relayUpdateInterval)

            return max(nextUpdate, Date())
        }

        private func handleResponse(
            completion: OperationCompletion<REST.ServerRelaysCacheResponse, REST.Error>
        ) -> OperationCompletion<FetchResult, Error> {
            let mappedCompletion = completion.tryMap { response -> FetchResult in
                switch response {
                case let .newContent(etag, relays):
                    try self.storeResponse(etag: etag, relays: relays)

                    return .newContent

                case .notModified:
                    return .sameContent
                }
            }

            if let error = mappedCompletion.error {
                logger.error(
                    error: error,
                    message: "Failed to update relays."
                )
            }

            return mappedCompletion
        }

        private func storeResponse(etag: String?, relays: REST.ServerRelaysResponse) throws {
            let numRelays = relays.wireguard.relays.count

            logger.info("Downloaded \(numRelays) relays.")

            let newCachedRelays = RelayCache.CachedRelays(
                etag: etag,
                relays: relays,
                updatedAt: Date()
            )

            nslock.lock()
            cachedRelays = newCachedRelays
            nslock.unlock()

            try RelayCache.IO.write(
                cacheFileURL: cacheFileURL,
                record: newCachedRelays
            )

            DispatchQueue.main.async {
                self.observerList.forEach { observer in
                    observer.relayCache(self, didUpdateCachedRelays: newCachedRelays)
                }
            }
        }

        private func scheduleRepeatingTimer(startTime: DispatchWallTime) {
            let timerSource = DispatchSource.makeTimerSource()
            timerSource.setEventHandler { [weak self] in
                _ = self?.updateRelays()
            }

            timerSource.schedule(
                wallDeadline: startTime,
                repeating: .seconds(Int(Self.relayUpdateInterval))
            )
            timerSource.activate()

            self.timerSource = timerSource
        }
    }
}
