//
//  Tunnel+Settings.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-06-19.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import MullvadLogging
import MullvadSettings

protocol TunnelSettingsStrategyProtocol {
    func shouldReconnectToNewRelay(oldSettings: LatestTunnelSettings, newSettings: LatestTunnelSettings) -> Bool
}

struct TunnelSettingsStrategy: TunnelSettingsStrategyProtocol {
    func shouldReconnectToNewRelay(oldSettings: LatestTunnelSettings, newSettings: LatestTunnelSettings) -> Bool {
        switch (oldSettings, newSettings) {
        case let (old, new) where old.relayConstraints != new.relayConstraints,
             let (old, new) where old.tunnelMultihopState != new.tunnelMultihopState:
            true
        default:
            false
        }
    }
}
