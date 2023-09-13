//
//  SettingsReaderProtocol.swift
//  PacketTunnel
//
//  Created by pronebird on 25/08/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings

public struct Settings {
    public var tunnelSettings: LatestTunnelSettings
    public var deviceState: DeviceState

    public init(tunnelSettings: LatestTunnelSettings, deviceState: DeviceState) {
        self.tunnelSettings = tunnelSettings
        self.deviceState = deviceState
    }
}

public protocol SettingsReaderProtocol {
    func read() throws -> Settings
}
