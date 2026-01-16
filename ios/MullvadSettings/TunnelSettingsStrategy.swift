//
//  TunnelSettingsStrategy.swift
//  MullvadSettings
//
//  Created by Mojgan on 2024-08-09.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

public protocol TunnelSettingsStrategyProtocol: Sendable {
    func shouldReconnectToNewRelay(oldSettings: LatestTunnelSettings, newSettings: LatestTunnelSettings) -> Bool
    func getReconnectionStrategy(
        oldSettings: LatestTunnelSettings,
        newSettings: LatestTunnelSettings
    ) -> TunnelSettingsReconnectionStrategy
}

public struct TunnelSettingsStrategy: TunnelSettingsStrategyProtocol, Sendable {
    public init() {}

    public func shouldReconnectToNewRelay(
        oldSettings: LatestTunnelSettings,
        newSettings: LatestTunnelSettings
    ) -> Bool {
        getReconnectionStrategy(
            oldSettings: oldSettings,
            newSettings: newSettings
        ) != .currentRelayReconnect
    }

    public func getReconnectionStrategy(
        oldSettings: LatestTunnelSettings,
        newSettings: LatestTunnelSettings
    ) -> TunnelSettingsReconnectionStrategy {
        // Don't reconnect the tunnel If IAN consent was the setting that triggered the settings update.
        if oldSettings.includeAllNetworks.consent != newSettings.includeAllNetworks.consent {
            return .noReconnect
        }

        if oldSettings.includeAllNetworks.localNetworkSharingState
            != newSettings.includeAllNetworks.localNetworkSharingState
            || oldSettings.includeAllNetworks.includeAllNetworksState
                != newSettings.includeAllNetworks.includeAllNetworksState
        {
            return .hardReconnect
        }

        switch (oldSettings, newSettings) {
        case let (old, new) where old != new:
            return .newRelayReconnect
        default:
            return .currentRelayReconnect
        }
    }
}

/// This enum represents reconnection strategies.
/// > Warning: `hardReconnect` will disconnect and reconnect which
/// > potentially leads to traffic leaking outside the tunnel.
public enum TunnelSettingsReconnectionStrategy {
    case currentRelayReconnect
    case newRelayReconnect
    case hardReconnect
    case noReconnect
}
