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
    let logger: Logger?

    init(logger: Logger? = nil) {
        self.logger = logger
    }

    func shouldReconnectToNewRelay(oldSettings: LatestTunnelSettings, newSettings: LatestTunnelSettings) -> Bool {
        switch (oldSettings, newSettings) {
        case let (old, new) where old.relayConstraints != new.relayConstraints:
            logger?.debug("Relay address changed from \(old.relayConstraints) to \(new.relayConstraints)")
            return true
        case let (old, new) where old.tunnelMultihopState != new.tunnelMultihopState:
            logger?
                .debug("Tunnel multi-hop state changed from \(old.tunnelMultihopState) to \(new.tunnelMultihopState)")
            return true
        default:
            return false
        }
    }
}
