//
//  RelayCacheTracker.swift
//  MullvadVPN
//
//  Created by pronebird on 05/06/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import BackgroundTasks
import Foundation
import Logging
import UIKit

extension RelayCache {

    class Tracker {
        /// Relay update interval (in seconds)
        private static let relayUpdateInterval: TimeInterval = 60 * 60

        /// Tracker log
        private let logger = Logger(label: "RelayCacheTracker")

        /// The cache location used by the class instance
        private let cacheFileURL: URL

        /// The location of prebundled `relays.json`
        private let prebundledRelaysFileURL: URL

        /// A dispatch queue used for thread synchronization
        private let stateQueue = DispatchQueue(label: "RelayCacheTrackerStateQueue")

        /// Internal operation queue.
        private let operationQueue: OperationQueue = {
            let operationQueue = AsyncOperationQueue()
            operationQueue.name = "RelayCacheTrackerQueue"
            operationQueue.maxConcurrentOperationCount = 1
            return operationQueue
        }()

        /// A timer source used for periodic updates
        private var timerSource: DispatchSourceTimer?

        /// A flag that indicates whether periodic updates are running
        private var isPeriodicUpdatesEnabled = false

        /// Observers
        private let observerList = ObserverList<RelayCacheObserver>()

        /// A shared instance of `RelayCache`
        static let shared: RelayCache.Tracker = {
            let cacheFileURL = RelayCache.IO.defaultCacheFileURL(forSecurityApplicationGroupIdentifier: ApplicationConfiguration.securityGroupIdentifier)!
            let prebundledRelaysFileURL = RelayCache.IO.preBundledRelaysFileURL!

            return Tracker(
                cacheFileURL: cacheFileURL,
                prebundledRelaysFileURL: prebundledRelaysFileURL
            )
        }()

        private init(cacheFileURL: URL, prebundledRelaysFileURL: URL) {
            self.cacheFileURL = cacheFileURL
            self.prebundledRelaysFileURL = prebundledRelaysFileURL
        }

        func startPeriodicUpdates() {
            stateQueue.async {
                guard !self.isPeriodicUpdatesEnabled else { return }

                self.logger.debug("Start periodic relay updates.")

                self.isPeriodicUpdatesEnabled = true

                do {
                    let cachedRelays = try RelayCache.IO.read(cacheFileURL: self.cacheFileURL)
                    let nextUpdate = cachedRelays.updatedAt
                        .addingTimeInterval(Self.relayUpdateInterval)

                    self.scheduleRepeatingTimer(startTime: .now() + nextUpdate.timeIntervalSinceNow)
                } catch {
                    self.logger.error(
                        chainedError: AnyChainedError(error),
                        message: "Failed to read the relay cache."
                    )

                    if let readError = error as? RelayCache.Error,
                       Self.shouldDownloadRelaysOnReadFailure(readError) {
                        self.scheduleRepeatingTimer(startTime: .now())
                    }
                }
            }
        }

        func stopPeriodicUpdates() {
            stateQueue.async {
                guard self.isPeriodicUpdatesEnabled else { return }

                self.logger.debug("Stop periodic relay updates.")

                self.isPeriodicUpdatesEnabled = false

                self.timerSource?.cancel()
                self.timerSource = nil
            }
        }

        func updateRelays(
            completionHandler: @escaping (
                OperationCompletion<RelayCache.FetchResult, RelayCache.Error>
            ) -> Void
        ) -> Cancellable {
            let operation = UpdateRelaysOperation(
                dispatchQueue: stateQueue,
                apiProxy: REST.ProxyFactory.shared.createAPIProxy(),
                cacheFileURL: self.cacheFileURL,
                relayUpdateInterval: Self.relayUpdateInterval,
                updateHandler: { [weak self] newCachedRelays in
                    guard let self = self else { return }

                    DispatchQueue.main.async {
                        self.observerList.forEach { observer in
                            observer.relayCache(self, didUpdateCachedRelays: newCachedRelays)
                        }
                    }
                },
                completionHandler: completionHandler
            )

            let backgroundTaskIdentifier = UIApplication.shared.beginBackgroundTask(withName: "Update relays") {
                operation.cancel()
            }

            operation.completionBlock = {
                UIApplication.shared.endBackgroundTask(backgroundTaskIdentifier)
            }

            operationQueue.addOperation(operation)

            return operation
        }

        func read(completionHandler: @escaping (Result<CachedRelays, RelayCache.Error>) -> Void) {
            stateQueue.async {
                let result = Result {
                    try RelayCache.IO.readWithFallback(
                        cacheFileURL: self.cacheFileURL,
                        preBundledRelaysFileURL: self.prebundledRelaysFileURL
                    )
                }.mapError { error in
                    return error as! RelayCache.Error
                }

                completionHandler(result)
            }
        }

        func readAndWait() throws -> CachedRelays {
            return try stateQueue.sync {
                return try RelayCache.IO.readWithFallback(
                    cacheFileURL: cacheFileURL,
                    preBundledRelaysFileURL: prebundledRelaysFileURL
                )
            }
        }

        // MARK: - Observation

        func addObserver(_ observer: RelayCacheObserver) {
            observerList.append(observer)
        }

        func removeObserver(_ observer: RelayCacheObserver) {
            observerList.remove(observer)
        }

        // MARK: - Private instance methods

        private func scheduleRepeatingTimer(startTime: DispatchWallTime) {
            let timerSource = DispatchSource.makeTimerSource(queue: stateQueue)
            timerSource.setEventHandler { [weak self] in
                _ = self?.updateRelays(completionHandler: { _ in
                    // no-op
                })
            }

            timerSource.schedule(wallDeadline: startTime, repeating: .seconds(Int(Self.relayUpdateInterval)))
            timerSource.activate()

            self.timerSource = timerSource
        }

        // MARK: - Private class methods

        private class func shouldDownloadRelaysOnReadFailure(_ error: RelayCache.Error) -> Bool {
            switch error {
            case .readPrebundledRelays, .decodePrebundledRelays, .decodeCache:
                return true

            case .readCache(CocoaError.fileReadNoSuchFile):
                return true

            default:
                return false
            }
        }
    }

}

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

}

// MARK: - Background tasks

@available(iOS 13.0, *)
extension RelayCache.Tracker {

    /// Register app refresh task with scheduler.
    func registerAppRefreshTask() {
        let taskIdentifier = ApplicationConfiguration.appRefreshTaskIdentifier

        let isRegistered = BGTaskScheduler.shared.register(forTaskWithIdentifier: taskIdentifier, using: nil) { task in
            self.handleAppRefreshTask(task as! BGAppRefreshTask)
        }

        if isRegistered {
            logger.debug("Registered app refresh task.")
        } else {
            logger.error("Failed to register app refresh task.")
        }
    }

    /// Schedules app refresh task relative to the last relays update.
    func scheduleAppRefreshTask() throws {
        let cachedRelays = try readAndWait()
        let beginDate = cachedRelays.updatedAt.addingTimeInterval(Self.relayUpdateInterval)

        try submitAppRefreshTask(at: beginDate)
    }

    /// Create and submit task request to scheduler.
    private func submitAppRefreshTask(at beginDate: Date) throws {
        let taskIdentifier = ApplicationConfiguration.appRefreshTaskIdentifier

        let request = BGAppRefreshTaskRequest(identifier: taskIdentifier)
        request.earliestBeginDate = beginDate

        try BGTaskScheduler.shared.submit(request)
    }

    /// Background task handler
    private func handleAppRefreshTask(_ task: BGAppRefreshTask) {
        logger.debug("Start app refresh task.")

        let cancellable = updateRelays { completion in
            switch completion {
            case .success(let fetchResult):
                self.logger.debug("Finished updating relays in app refresh task: \(fetchResult).")

            case .failure(let error):
                self.logger.error(
                    chainedError: error,
                    message: "Failed to update relays in app refresh task."
                )

            case .cancelled:
                self.logger.debug("App refresh task was cancelled.")
            }

            task.setTaskCompleted(success: completion.isSuccess)
        }

        task.expirationHandler = {
            cancellable.cancel()
        }

        // Schedule next refresh
        let scheduleDate = Date(timeIntervalSinceNow: Self.relayUpdateInterval)
        do {
            try submitAppRefreshTask(at: scheduleDate)

            logger.debug("Scheduled next app refresh task at \(scheduleDate.logFormatDate()).")
        } catch {
            logger.error(
                chainedError: AnyChainedError(error),
                message: "Failed to schedule next app refresh task."
            )
        }
    }
}

fileprivate class UpdateRelaysOperation: ResultOperation<RelayCache.FetchResult, RelayCache.Error> {
    typealias UpdateHandler = (RelayCache.CachedRelays) -> Void

    private let apiProxy: REST.APIProxy
    private let cacheFileURL: URL
    private let relayUpdateInterval: TimeInterval

    private let logger = Logger(label: "RelayCacheTracker.UpdateRelaysOperation")

    private let updateHandler: UpdateHandler
    private var downloadTask: Cancellable?

    init(
        dispatchQueue: DispatchQueue,
        apiProxy: REST.APIProxy,
        cacheFileURL: URL,
        relayUpdateInterval: TimeInterval,
        updateHandler: @escaping UpdateHandler,
        completionHandler: @escaping CompletionHandler
    )
    {
        self.apiProxy = apiProxy
        self.cacheFileURL = cacheFileURL
        self.relayUpdateInterval = relayUpdateInterval
        self.updateHandler = updateHandler

        super.init(
            dispatchQueue: dispatchQueue,
            completionQueue: dispatchQueue,
            completionHandler: completionHandler
        )
    }

    override func main() {
        do {
            let cachedRelays = try RelayCache.IO.read(cacheFileURL: cacheFileURL)
            let nextUpdate = cachedRelays.updatedAt.addingTimeInterval(relayUpdateInterval)

            if nextUpdate <= Date() {
                downloadRelays(previouslyCachedRelays: cachedRelays)
            } else {
                finish(completion: .success(.throttled))
            }
        } catch {
            let error = error as! RelayCache.Error

            logger.error(
                chainedError: error,
                message: "Failed to read the relay cache to determine if it needs to be updated."
            )

            if shouldDownloadRelaysOnReadFailure(error) {
                downloadRelays(previouslyCachedRelays: nil)
            } else {
                finish(completion: .failure(error))
            }
        }
    }

    override func operationDidCancel() {
        downloadTask?.cancel()
        downloadTask = nil
    }

    private func didReceiveNewRelays(etag: String?, relays: REST.ServerRelaysResponse) {
        let numRelays = relays.wireguard.relays.count

        logger.info("Downloaded \(numRelays) relays.")

        let cachedRelays = RelayCache.CachedRelays(
            etag: etag,
            relays: relays,
            updatedAt: Date()
        )

        do {
            try RelayCache.IO.write(cacheFileURL: cacheFileURL, record: cachedRelays)

            updateHandler(cachedRelays)

            finish(completion: .success(.newContent))
        } catch {
            let error = error as! RelayCache.Error

            logger.error(
                chainedError: error,
                message: "Failed to store downloaded relays."
            )

            finish(completion: .failure(error))
        }
    }

    private func didReceiveNotModified(previouslyCachedRelays: RelayCache.CachedRelays) {
        var cachedRelays = previouslyCachedRelays
        cachedRelays.updatedAt = Date()

        logger.info("Relays haven't changed since last check.")

        do {
            try RelayCache.IO.write(cacheFileURL: cacheFileURL, record: cachedRelays)

            finish(completion: .success(.sameContent))
        } catch {
            let error = error as! RelayCache.Error

            logger.error(
                chainedError: error,
                message: "Failed to update cached relays timestamp."
            )

            finish(completion: .failure(error))
        }
    }

    private func didFailToDownloadRelays(error: REST.Error) {
        logger.error(chainedError: error, message: "Failed to download relays.")

        finish(completion: .failure(.rest(error)))
    }

    private func downloadRelays(previouslyCachedRelays: RelayCache.CachedRelays?) {
        downloadTask = apiProxy.getRelays(etag: previouslyCachedRelays?.etag, retryStrategy: .noRetry) { [weak self] completion in
            guard let self = self else { return }

            self.dispatchQueue.async {
                switch completion {
                case .success(.newContent(let etag, let relays)):
                    self.didReceiveNewRelays(etag: etag, relays: relays)

                case .success(.notModified):
                    self.didReceiveNotModified(previouslyCachedRelays: previouslyCachedRelays!)

                case .failure(let error):
                    self.didFailToDownloadRelays(error: error)

                case .cancelled:
                    self.logger.debug("Cancelled relays download.")
                    self.finish(completion: .cancelled)
                }
            }
        }
    }

    private func shouldDownloadRelaysOnReadFailure(_ error: RelayCache.Error) -> Bool {
        switch error {
        case .readPrebundledRelays, .decodePrebundledRelays, .decodeCache:
            return true

        case .readCache(CocoaError.fileReadNoSuchFile):
            return true

        default:
            return false
        }
    }
}
