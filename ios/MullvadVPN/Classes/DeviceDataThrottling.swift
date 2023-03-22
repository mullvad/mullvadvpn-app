//
//  DeviceDataThrottling.swift
//  MullvadVPN
//
//  Created by pronebird on 13/12/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Struct used for throttling UI calls to update device data via tunnel manager.
struct DeviceDataThrottling {
    /// Default cooldown interval between requests.
    private static let defaultWaitInterval: TimeInterval = 60

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

        let nextUpdateAfter = lastUpdate?.addingTimeInterval(Self.defaultWaitInterval)
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
        tunnelManager.updateDeviceData()
    }
}
