//
//  AddressCacheUpdateScheduler.swift
//  MullvadVPN
//
//  Created by pronebird on 08/12/2021.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadLogging
import MullvadREST
import MullvadRustRuntime
import MullvadTypes
import Operations
import UIKit

final class AddressCacheUpdateScheduler: @unchecked Sendable {
    /// Update interval.
    private static let updateInterval: Duration = .days(1)

    /// Retry interval.
    private static let retryInterval: Duration = .minutes(15)

    /// Logger.
    private let logger = Logger(label: "AddressCache.Tracker")
    private let backgroundTaskProvider: BackgroundTaskProviding

    /// REST API proxy.
    private let apiProxy: APIQuerying

    /// A flag that indicates whether periodic updates are running
    private var isPeriodicUpdatesEnabled = false

    /// The date of last failed attempt.
    private var lastFailureAttemptDate: Date?

    /// The timestamp of the last update request
    private var lastUpdateRequestDate: Date?

    /// Timer used for scheduling periodic updates.
    private var timer: DispatchSourceTimer?

    /// Operation queue.
    private let operationQueue = AsyncOperationQueue.makeSerial()

    /// Lock used for synchronizing member access.
    private let nslock = NSLock()

    private let apiContext: MullvadApiContext

    /// Designated initializer
    init(backgroundTaskProvider: BackgroundTaskProviding, apiProxy: APIQuerying, apiContext: MullvadApiContext) {
        self.backgroundTaskProvider = backgroundTaskProvider
        self.apiProxy = apiProxy
        self.apiContext = apiContext
    }

    func startPeriodicUpdates() {
        nslock.withLock {
            guard !isPeriodicUpdatesEnabled else {
                return
            }

            logger.debug("Start periodic address cache updates.")

            isPeriodicUpdatesEnabled = true

            let scheduleDate = _nextScheduleDate()

            logger.debug("Schedule address cache update at \(scheduleDate.logFormatted).")

            scheduleEndpointsUpdate(startTime: .now() + scheduleDate.timeIntervalSinceNow)
        }
    }

    func stopPeriodicUpdates() {
        nslock.withLock {
            guard isPeriodicUpdatesEnabled else { return }

            logger.debug("Stop periodic address cache updates.")

            isPeriodicUpdatesEnabled = false

            timer?.cancel()
            timer = nil
        }
    }

    func updateEndpoints(completionHandler: ((sending Result<Bool, Error>) -> Void)? = nil) {
        mullvad_api_update_address_cache(apiContext.context)
        recordUpdateRequestTime()
    }

    func nextScheduleDate() -> Date {
        nslock.withLock {
            _nextScheduleDate()
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
        updateEndpoints { [weak self] _ in
            guard let self else { return }
            self.nslock.withLock {
                guard self.isPeriodicUpdatesEnabled else { return }

                let scheduleDate = self._nextScheduleDate()

                self.logger
                    .debug("Schedule next address cache update at \(scheduleDate.logFormatted).")

                self.scheduleEndpointsUpdate(startTime: .now() + scheduleDate.timeIntervalSinceNow)
            }
        }
    }

    private func _nextScheduleDate() -> Date {
        if let lastUpdateRequestDate {
            max(Date(timeInterval: Self.retryInterval.timeInterval, since: lastUpdateRequestDate), Date())
        } else {
            Date()
        }
    }

    private func recordUpdateRequestTime() {
        nslock.withLock {
            lastUpdateRequestDate = Date()
        }
    }
}
