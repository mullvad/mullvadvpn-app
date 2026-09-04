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

/// Struct used for throttling UI calls to update device data via tunnel manager.
struct DeviceDataThrottling {
    /// Default cooldown interval between requests.
    private static let defaultWaitInterval: Duration = .minutes(1)

    let tunnelManager: TunnelManager
    private(set) var lastUpdate: Date?

    init(tunnelManager: TunnelManager) {
        self.tunnelManager = tunnelManager
    }

    mutating func requestUpdate(forceUpdate: Bool) {
        guard tunnelManager.deviceState.isLoggedIn else {
            return
        }

        let now = Date()

        guard !forceUpdate else {
            startUpdate(now: now)
            return
        }

        let nextUpdateAfter = lastUpdate?.addingTimeInterval(Self.defaultWaitInterval.timeInterval)
        let comparisonResult = nextUpdateAfter?.compare(now) ?? .orderedAscending

        switch comparisonResult {
        case .orderedAscending, .orderedSame:
            startUpdate(now: now)

        case .orderedDescending:
            break
        }
    }

    mutating func reset() {
        lastUpdate = nil
    }

    private mutating func startUpdate(now: Date) {
        lastUpdate = now
        Task { [tunnelManager] in
            try? await tunnelManager.updateDeviceData()
        }
    }
}
