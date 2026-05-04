//
//  DeviceStateAccessor.swift
//  PacketTunnel
//
//  Created by pronebird on 30/05/2023.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings
import MullvadTypes

/// An object that provides access to `DeviceState` used by `DeviceCheckOperation`.
struct DeviceStateAccessor: DeviceStateAccessorProtocol {
    let settingsManager: SettingsManager

    init(settingsManager: SettingsManager) {
        self.settingsManager = settingsManager
    }

    func read() throws -> DeviceState {
        try settingsManager.readDeviceState()
    }

    func write(_ deviceState: DeviceState) throws {
        try settingsManager.writeDeviceState(deviceState)
    }
}
