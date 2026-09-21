// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Logging
import MullvadTypes

struct AccountDataPoller {
    private let logger: Logger
    private let accountUpdateTimerInterval: Duration
    private let accountUpdateTimer: any DispatchSourceTimer

    init(
        logger: Logger,
        tunnelManager: TunnelManager,
        accountUpdateTimerInterval: Duration = .seconds(15),
    ) {
        self.logger = logger
        self.accountUpdateTimerInterval = accountUpdateTimerInterval
        accountUpdateTimer = DispatchSource.makeTimerSource(queue: .main)
        accountUpdateTimer.setEventHandler { [weak tunnelManager] in
            Task {
                try? await tunnelManager?.updateAccountData()
            }
        }
    }

    func startAccountUpdateTimer() {
        logger.debug(
            "Start polling account updates every \(accountUpdateTimerInterval) second(s)."
        )
        accountUpdateTimer.schedule(
            wallDeadline: .now() + accountUpdateTimerInterval,
            repeating: accountUpdateTimerInterval.timeInterval
        )
        accountUpdateTimer.activate()
    }

    func stopAccountUpdateTimer() {
        logger.debug(
            "Stop polling account updates."
        )

        accountUpdateTimer.cancel()
    }
}
