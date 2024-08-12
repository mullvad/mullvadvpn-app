//
//  TunnelSettingsStrategy.swift
//  MullvadSettings
//
//  Created by Mojgan on 2024-08-09.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
public protocol TunnelSettingsStrategyProtocol {
    func shouldReconnectToNewRelay(oldSettings: LatestTunnelSettings, newSettings: LatestTunnelSettings) -> Bool
}

public struct TunnelSettingsStrategy: TunnelSettingsStrategyProtocol {
    public init() {}
    public func shouldReconnectToNewRelay(
        oldSettings: LatestTunnelSettings,
        newSettings: LatestTunnelSettings
    ) -> Bool {
        switch (oldSettings, newSettings) {
        case let (old, new) where old.relayConstraints != new.relayConstraints,
             let (old, new) where old.tunnelMultihopState != new.tunnelMultihopState,
             let (old, new) where old.daita != new.daita:
            true
        default:
            false
        }
    }
}
