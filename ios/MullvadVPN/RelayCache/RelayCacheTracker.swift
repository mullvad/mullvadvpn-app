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
            let operationQueue = OperationQueue()
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

                switch RelayCache.IO.read(cacheFileURL: self.cacheFileURL) {
                case .success(let cachedRelays):
                    let nextUpdate = cachedRelays.updatedAt.addingTimeInterval(Self.relayUpdateInterval)
                    self.scheduleRepeatingTimer(startTime: .now() + nextUpdate.timeIntervalSinceNow)

                case .failure(let readError):
                    self.logger.error(chainedError: readError, message: "Failed to read the relay cache.")

                    if Self.shouldDownloadRelaysOnReadFailure(readError) {
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

        func updateRelays(completionHandler: @escaping (OperationCompletion<RelayCache.FetchResult, RelayCache.Error>) -> Void) -> AnyCancellable {
            let operation = UpdateRelaysOperation(
                dispatchQueue: stateQueue,
                restClient: REST.Client.shared,
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

            return AnyCancellable {
                operation.cancel()
            }
        }

        func read(completionHandler: @escaping (Result<CachedRelays, RelayCache.Error>) -> Void) {
            stateQueue.async {
                let result = RelayCache.IO.readWithFallback(
                    cacheFileURL: self.cacheFileURL,
                    preBundledRelaysFileURL: self.prebundledRelaysFileURL
                )

                completionHandler(result)
            }
        }

        func readAndWait() -> Result<CachedRelays, RelayCache.Error> {
            return stateQueue.sync {
                return RelayCache.IO.readWithFallback(
                    cacheFileURL: self.cacheFileURL,
                    preBundledRelaysFileURL: self.prebundledRelaysFileURL
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
    func scheduleAppRefreshTask() -> Result<(), RelayCache.Error> {
        return readAndWait().flatMap { cachedRelays in
            let beginDate = cachedRelays.updatedAt.addingTimeInterval(Self.relayUpdateInterval)

            return self.submitAppRefreshTask(at: beginDate)
        }
    }

    /// Create and submit task request to scheduler.
    private func submitAppRefreshTask(at beginDate: Date) -> Result<(), RelayCache.Error> {
        let taskIdentifier = ApplicationConfiguration.appRefreshTaskIdentifier

        let request = BGAppRefreshTaskRequest(identifier: taskIdentifier)
        request.earliestBeginDate = beginDate

        return Result { try BGTaskScheduler.shared.submit(request) }
            .mapError { error in
                return .backgroundTaskScheduler(error)
            }
    }

    /// Background task handler
    private func handleAppRefreshTask(_ task: BGAppRefreshTask) {
        logger.debug("Start app refresh task.")

        let cancellable = self.updateRelays { completion in
            let isTaskCompleted: Bool

            switch completion {
            case .success(let fetchResult):
                self.logger.debug("Finished updating relays in app refresh task: \(fetchResult).")
                isTaskCompleted = true

            case .failure(let error):
                self.logger.error(chainedError: error, message: "Failed to update relays in app refresh task.")
                isTaskCompleted = false

            case .cancelled:
                self.logger.debug("App refresh task was cancelled.")
                isTaskCompleted = false
            }

            task.setTaskCompleted(success: isTaskCompleted)
        }

        task.expirationHandler = {
            cancellable.cancel()
        }

        // Schedule next refresh
        let scheduleDate = Date(timeIntervalSinceNow: Self.relayUpdateInterval)

        switch self.submitAppRefreshTask(at: scheduleDate) {
        case .success:
            logger.debug("Scheduled next app refresh task at \(scheduleDate.logFormatDate()).")

        case .failure(let error):
            logger.error(chainedError: error, message: "Failed to schedule next app refresh task.")
        }
    }
}

fileprivate class UpdateRelaysOperation: AsyncOperation {
    typealias UpdateHandler = (RelayCache.CachedRelays) -> Void
    typealias CompletionHandler = (OperationCompletion<RelayCache.FetchResult, RelayCache.Error>) -> Void

    private let dispatchQueue: DispatchQueue
    private let restClient: REST.Client
    private let cacheFileURL: URL
    private let relayUpdateInterval: TimeInterval

    private let logger = Logger(label: "RelayCacheTracker.UpdateRelaysOperation")

    private let updateHandler: UpdateHandler
    private var completionHandler: CompletionHandler?
    private var downloadCancellable: AnyCancellable?

    init(dispatchQueue: DispatchQueue,
         restClient: REST.Client,
         cacheFileURL: URL,
         relayUpdateInterval: TimeInterval,
         updateHandler: @escaping UpdateHandler,
         completionHandler: @escaping CompletionHandler) {
        self.dispatchQueue = dispatchQueue
        self.restClient = restClient
        self.cacheFileURL = cacheFileURL
        self.relayUpdateInterval = relayUpdateInterval
        self.updateHandler = updateHandler
        self.completionHandler = completionHandler
    }

    override func main() {
        dispatchQueue.async {
            guard !self.isCancelled else {
                self.finish(completion: .cancelled)
                return
            }

            let readResult = RelayCache.IO.read(cacheFileURL: self.cacheFileURL)
            switch readResult {
            case .success(let cachedRelays):
                let nextUpdate = cachedRelays.updatedAt.addingTimeInterval(self.relayUpdateInterval)

                if nextUpdate <= Date() {
                    self.downloadRelays(previouslyCachedRelays: cachedRelays)
                } else {
                    self.finish(completion: .success(.throttled))
                }

            case .failure(let readError):
                self.logger.error(chainedError: readError, message: "Failed to read the relay cache to determine if it needs to be updated.")

                if self.shouldDownloadRelaysOnReadFailure(readError) {
                    self.downloadRelays(previouslyCachedRelays: nil)
                } else {
                    self.finish(completion: .failure(readError))
                }
            }
        }
    }

    override func cancel() {
        super.cancel()

        dispatchQueue.async {
            self.downloadCancellable?.cancel()
        }
    }

    private func finish(completion: OperationCompletion<RelayCache.FetchResult, RelayCache.Error>) {
        let block = completionHandler
        completionHandler = nil

        block?(completion)

        finish()
    }

    private func didReceiveNewRelays(etag: String?, relays: REST.ServerRelaysResponse) {
        let numRelays = relays.wireguard.relays.count

        logger.info("Downloaded \(numRelays) relays.")

        let cachedRelays = RelayCache.CachedRelays(etag: etag, relays: relays, updatedAt: Date())
        let writeResult = RelayCache.IO.write(cacheFileURL: cacheFileURL, record: cachedRelays)

        switch writeResult {
        case .success:
            updateHandler(cachedRelays)

            finish(completion: .success(.newContent))

        case .failure(let error):
            logger.error(chainedError: error, message: "Failed to store downloaded relays.")

            finish(completion: .failure(.writeCache(error)))
        }
    }

    private func didReceiveNotModified(previouslyCachedRelays: RelayCache.CachedRelays) {
        logger.info("Relays haven't changed since last check.")

        var cachedRelays = previouslyCachedRelays
        cachedRelays.updatedAt = Date()

        let writeResult = RelayCache.IO.write(cacheFileURL: self.cacheFileURL, record: cachedRelays)

        switch writeResult {
        case .success:
            finish(completion: .success(.sameContent))

        case .failure(let error):
            logger.error(chainedError: error, message: "Failed to update cached relays timestamp.")

            finish(completion: .failure(.writeCache(error)))
        }
    }

    private func didFailToDownloadRelays(error: REST.Error) {
        logger.error(chainedError: error, message: "Failed to download relays.")

        finish(completion: .failure(.rest(error)))
    }

    private func downloadRelays(previouslyCachedRelays: RelayCache.CachedRelays?) {
        downloadCancellable = REST.Client.shared.getRelays(etag: previouslyCachedRelays?.etag, retryStrategy: .noRetry) { [weak self] result in
            guard let self = self else { return }

            self.dispatchQueue.async {
                switch result {
                case .success(.newContent(let etag, let relays)):
                    self.didReceiveNewRelays(etag: etag, relays: relays)

                case .success(.notModified):
                    self.didReceiveNotModified(previouslyCachedRelays: previouslyCachedRelays!)

                case .failure(let error):
                    self.didFailToDownloadRelays(error: error)
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
