//
//  AddressCacheTracker.swift
//  MullvadVPN
//
//  Created by pronebird on 08/12/2021.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadLogging
import MullvadREST
import MullvadTypes
import Operations
import UIKit

final class AddressCacheTracker: @unchecked Sendable {
    /// Update interval.
    private static let updateInterval: Duration = .days(1)

    /// Retry interval.
    private static let retryInterval: Duration = .minutes(15)

    /// Logger.
    private let logger = Logger(label: "AddressCache.Tracker")
    private let backgroundTaskProvider: BackgroundTaskProviding

    /// REST API proxy.
    private let apiProxy: APIQuerying

    /// Address cache.
    private let store: REST.AddressCache

    /// A flag that indicates whether periodic updates are running
    private var isPeriodicUpdatesEnabled = false

    /// The date of last failed attempt.
    private var lastFailureAttemptDate: Date?

    /// Timer used for scheduling periodic updates.
    private var timer: DispatchSourceTimer?

    /// Operation queue.
    private let operationQueue = AsyncOperationQueue.makeSerial()

    /// Lock used for synchronizing member access.
    private let nslock = NSLock()

    /// Designated initializer
    init(backgroundTaskProvider: BackgroundTaskProviding, apiProxy: APIQuerying, store: REST.AddressCache) {
        self.backgroundTaskProvider = backgroundTaskProvider
        self.apiProxy = apiProxy
        self.store = store
    }

    func startPeriodicUpdates() {
        nslock.lock()
        defer { nslock.unlock() }

        guard !isPeriodicUpdatesEnabled else {
            return
        }

        logger.debug("Start periodic address cache updates.")

        isPeriodicUpdatesEnabled = true

        let scheduleDate = _nextScheduleDate()

        logger.debug("Schedule address cache update at \(scheduleDate.logFormatted).")

        scheduleEndpointsUpdate(startTime: .now() + scheduleDate.timeIntervalSinceNow)
    }

    func stopPeriodicUpdates() {
        nslock.lock()
        defer { nslock.unlock() }

        guard isPeriodicUpdatesEnabled else { return }

        logger.debug("Stop periodic address cache updates.")

        isPeriodicUpdatesEnabled = false

        timer?.cancel()
        timer = nil
    }

    func updateEndpoints(completionHandler: ((sending Result<Bool, Error>) -> Void)? = nil) -> Cancellable {
        let operation = ResultBlockOperation<Bool> { finish -> Cancellable in
            guard self.nextScheduleDate() <= Date() else {
                finish(.success(false))
                return AnyCancellable()
            }

            return self.apiProxy.mullvadApiGetAddressList(retryStrategy: .default) { result in
                print("Address list: \(result)")
                self.setEndpoints(from: result)
                finish(result.map { _ in true })
            }
        }

        operation.completionQueue = .main
        operation.completionHandler = completionHandler

        operation.addObserver(
            BackgroundObserver(
                backgroundTaskProvider: backgroundTaskProvider,
                name: "Update endpoints",
                cancelUponExpiration: true
            )
        )

        operationQueue.addOperation(operation)

        return operation
    }

    func nextScheduleDate() -> Date {
        nslock.lock()
        defer { nslock.unlock() }

        return _nextScheduleDate()
    }

    private func setEndpoints(from result: Result<[AnyIPEndpoint], Error>) {
        nslock.lock()
        defer { nslock.unlock() }

        switch result {
        case let .success(endpoints):
            store.setEndpoints(endpoints)
            lastFailureAttemptDate = nil

        case let .failure(error as REST.Error):
            logger.error(
                error: error,
                message: "Failed to update address cache."
            )
            fallthrough

        default:
            lastFailureAttemptDate = Date()
        }
    }

    private func scheduleEndpointsUpdate(startTime: DispatchWallTime) {
        let newTimer = DispatchSource.makeTimerSource()
        newTimer.setEventHandler { [weak self] in
            self?.handleTimer()
        }

        newTimer.schedule(wallDeadline: startTime)
        newTimer.activate()

        timer?.cancel()
        timer = newTimer
    }

    private func handleTimer() {
        _ = updateEndpoints { _ in
            self.nslock.lock()
            defer { self.nslock.unlock() }

            guard self.isPeriodicUpdatesEnabled else { return }

            let scheduleDate = self._nextScheduleDate()

            self.logger
                .debug("Schedule next address cache update at \(scheduleDate.logFormatted).")

            self.scheduleEndpointsUpdate(startTime: .now() + scheduleDate.timeIntervalSinceNow)
        }
    }

    private func _nextScheduleDate() -> Date {
        let nextDate = lastFailureAttemptDate.map { date in
            Date(
                timeInterval: Self.retryInterval.timeInterval,
                since: date
            )
        } ?? Date(
            timeInterval: Self.updateInterval.timeInterval,
            since: store.getLastUpdateDate()
        )

        return max(nextDate, Date())
    }
}
