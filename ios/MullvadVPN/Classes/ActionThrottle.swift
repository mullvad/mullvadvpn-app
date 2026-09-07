// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation
import MullvadTypes

/// A mechanism for throttling an action, i.e., allowing only one request in a time interval.
/// The time interval can be dynamically determined by a computation at time of request

actor ActionThrottle {

    /// A function that returns the current wait interval. The current time is passed to it.
    let waitInterval: ((Date) -> Duration)
    /// The action to carry out.
    let action: (() async -> Void)

    init(
        waitInterval: @escaping @Sendable (Date) -> Duration,
        action: @escaping @Sendable () async -> Void
    ) {
        self.waitInterval = waitInterval
        self.action = action
    }

    init(
        waitInterval: Duration,
        action: @escaping @Sendable () async -> Void
    ) {
        self.waitInterval = { _ in waitInterval }
        self.action = action
    }

    private var lastUpdate: Date? = nil

    /// Request the action to be carried out. The action may or may not be carried out, depending on whether one was carried out within the interval.
    /// - Parameter force: if `true`, the action will be carried out regardless of whether the time interval has passed. The last timestamp will be updated in either case.
    func requestAction(force: Bool) async {
        let now = Date()

        guard let lastUpdate, !force else {
            run(now: now)
            return
        }

        let interval = waitInterval(now)
        let nextDue = lastUpdate + interval

        if now >= nextDue {
            run(now: now)
        }
    }

    private func run(now: Date) {
        lastUpdate = now
        Task {
            await action()
        }
    }

    /// Reset the interval timer.
    func reset() {
        lastUpdate = nil
    }
}
