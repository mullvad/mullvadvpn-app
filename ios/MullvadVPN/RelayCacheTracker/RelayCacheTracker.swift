//
//  RelayCacheTracker.swift
//  MullvadVPN
//
//  Created by pronebird on 05/06/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadREST
import MullvadTypes
import Operations
import UIKit

protocol RelayCacheTrackerProtocol {
    func startPeriodicUpdates()
    func stopPeriodicUpdates()
    func updateRelays(completionHandler: ((Result<RelaysFetchResult, Error>) -> Void)?) -> Cancellable
    func getCachedRelays() throws -> CachedRelays
    func getNextUpdateDate() -> Date
    func addObserver(_ observer: RelayCacheTrackerObserver)
    func removeObserver(_ observer: RelayCacheTrackerObserver)
    func refreshCachedRelays() throws
}

final class RelayCacheTracker: RelayCacheTrackerProtocol {
    /// Relay update interval.
    static let relayUpdateInterval: Duration = .hours(1)

    /// Tracker log.
    private let logger = Logger(label: "RelayCacheTracker")

    /// Relay cache.
    private let cache: RelayCacheProtocol

    private let application: UIApplication

    /// Lock used for synchronization.
    private let nslock = NSLock()

    /// Internal operation queue.
    private let operationQueue = AsyncOperationQueue.makeSerial()

    /// A timer source used for periodic updates.
    private var timerSource: DispatchSourceTimer?

    /// A flag that indicates whether periodic updates are running.
    private var isPeriodicUpdatesEnabled = false

    /// API proxy.
    private let apiProxy: APIQuerying

    /// Observers.
    private let observerList = ObserverList<RelayCacheTrackerObserver>()

    /// Memory cache.
    private var cachedRelays: CachedRelays?

    init(relayCache: RelayCacheProtocol, application: UIApplication, apiProxy: APIQuerying) {
        self.application = application
        self.apiProxy = apiProxy
        cache = relayCache

        do {
            cachedRelays = try cache.read()
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

    func updateRelays(completionHandler: ((Result<RelaysFetchResult, Error>) -> Void)? = nil)
        -> Cancellable {
        let operation = ResultBlockOperation<RelaysFetchResult> { finish in
            let cachedRelays = try? self.getCachedRelays()

            if self.getNextUpdateDate() > Date() {
                finish(.success(.throttled))
                return AnyCancellable()
            }

            return self.apiProxy.getRelays(etag: cachedRelays?.etag, retryStrategy: .noRetry) { result in
                finish(self.handleResponse(result: result))
            }
        }

        operation.addObserver(
            BackgroundObserver(
                application: application,
                name: "Update relays",
                cancelUponExpiration: true
            )
        )

        operation.completionQueue = .main
        operation.completionHandler = completionHandler

        operationQueue.addOperation(operation)

        return operation
    }

    func getCachedRelays() throws -> CachedRelays {
        nslock.lock()
        defer { nslock.unlock() }

        if let cachedRelays {
            return cachedRelays
        } else {
            throw NoCachedRelaysError()
        }
    }

    func refreshCachedRelays() throws {
        let newCachedRelays = try cache.read()

        nslock.lock()
        cachedRelays = newCachedRelays
        nslock.unlock()

        DispatchQueue.main.async {
            self.observerList.forEach { observer in
                observer.relayCacheTracker(self, didUpdateCachedRelays: newCachedRelays)
            }
        }
    }

    func getNextUpdateDate() -> Date {
        nslock.lock()
        defer { nslock.unlock() }

        return _getNextUpdateDate()
    }

    // MARK: - Observation

    func addObserver(_ observer: RelayCacheTrackerObserver) {
        observerList.append(observer)
    }

    func removeObserver(_ observer: RelayCacheTrackerObserver) {
        observerList.remove(observer)
    }

    // MARK: - Private

    private func _getNextUpdateDate() -> Date {
        let now = Date()

        guard let cachedRelays else {
            return now
        }

        let nextUpdate = cachedRelays.updatedAt.addingTimeInterval(Self.relayUpdateInterval.timeInterval)

        return max(nextUpdate, Date())
    }

    private func handleResponse(result: Result<REST.ServerRelaysCacheResponse, Error>)
        -> Result<RelaysFetchResult, Error> {
        result.tryMap { response -> RelaysFetchResult in
            switch response {
            case let .newContent(etag, relays):
                try self.storeResponse(etag: etag, relays: relays)

                return .newContent

            case .notModified:
                return .sameContent
            }
        }.inspectError { error in
            guard !error.isOperationCancellationError else { return }

            logger.error(
                error: error,
                message: "Failed to update relays."
            )
        }
    }

    private func storeResponse(etag: String?, relays: REST.ServerRelaysResponse) throws {
        let numRelays = relays.wireguard.relays.count

        logger.info("Downloaded \(numRelays) relays.")

        let newCachedRelays = CachedRelays(
            etag: etag,
            relays: relays,
            updatedAt: Date()
        )

        try cache.write(record: newCachedRelays)
        try refreshCachedRelays()
    }

    private func scheduleRepeatingTimer(startTime: DispatchWallTime) {
        let timerSource = DispatchSource.makeTimerSource()
        timerSource.setEventHandler { [weak self] in
            _ = self?.updateRelays()
        }

        timerSource.schedule(
            wallDeadline: startTime,
            repeating: Self.relayUpdateInterval.timeInterval
        )
        timerSource.activate()

        self.timerSource = timerSource
    }
}

/// Type describing the result of an attempt to fetch the new relay list from server.
enum RelaysFetchResult: CustomStringConvertible {
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
        "Relay cache is empty."
    }
}
