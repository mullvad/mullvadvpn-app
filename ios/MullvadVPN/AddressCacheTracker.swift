//
//  AddressCacheTracker.swift
//  MullvadVPN
//
//  Created by pronebird on 08/12/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import MullvadLogging
import MullvadREST
import MullvadTypes
import Operations
import UIKit

final class AddressCacheTracker {
    /// Update interval (in seconds).
    private static let updateInterval: TimeInterval = 60 * 60 * 24

    /// Retry interval (in seconds).
    private static let retryInterval: TimeInterval = 60 * 15

    /// Logger.
    private let logger = Logger(label: "AddressCache.Tracker")
    private let application: UIApplication

    /// REST API proxy.
    private let apiProxy: REST.APIProxy

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
    init(application: UIApplication, apiProxy: REST.APIProxy, store: REST.AddressCache) {
        self.application = application
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

        logger.debug("Schedule address cache update at \(scheduleDate.logFormatDate()).")

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

    func updateEndpoints(
        completionHandler: ((OperationCompletion<Bool, Error>) -> Void)? = nil
    ) -> Cancellable {
        let operation = ResultBlockOperation<Bool, Error> { operation in
            guard self.nextScheduleDate() <= Date() else {
                operation.finish(completion: .success(false))
                return
            }

            let task = self.apiProxy.getAddressList(retryStrategy: .default) { completion in
                self.setEndpoints(from: completion)

                let mappedCompletion = completion.map { _ in true }
                    .eraseFailureType()

                operation.finish(completion: mappedCompletion)
            }

            operation.addCancellationBlock {
                task.cancel()
            }
        }

        operation.completionQueue = .main
        operation.completionHandler = completionHandler

        operation.addObserver(
            BackgroundObserver(
                application: application,
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

    private func setEndpoints(from completion: OperationCompletion<[AnyIPEndpoint], REST.Error>) {
        nslock.lock()
        defer { nslock.unlock() }

        switch completion {
        case let .success(endpoints):
            store.setEndpoints(endpoints)

        case let .failure(error):
            logger.error(
                error: error,
                message: "Failed to update address cache."
            )

        case .cancelled:
            break
        }

        lastFailureAttemptDate = completion.isSuccess ? nil : Date()
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
                .debug("Schedule next address cache update at \(scheduleDate.logFormatDate()).")

            self.scheduleEndpointsUpdate(startTime: .now() + scheduleDate.timeIntervalSinceNow)
        }
    }

    private func _nextScheduleDate() -> Date {
        let nextDate = lastFailureAttemptDate.map { date in
            return Date(
                timeInterval: Self.retryInterval,
                since: date
            )
        } ?? Date(
            timeInterval: Self.updateInterval,
            since: store.getLastUpdateDate()
        )

        return max(nextDate, Date())
    }
}
