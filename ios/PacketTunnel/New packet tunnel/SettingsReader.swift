//
//  SettingsReader.swift
//  PacketTunnel
//
//  Created by pronebird on 30/08/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings
import PacketTunnelCore

struct SettingsReader: SettingsReaderProtocol {
    func read() throws -> Settings {
        return try Settings(
            tunnelSettings: SettingsManager.readSettings(),
            deviceState: SettingsManager.readDeviceState()
        )
    }
}
