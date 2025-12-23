//
//  TunnelSettingsStrategy.swift
//  MullvadSettings
//
//  Created by Mojgan on 2024-08-09.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
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
        // Hard reconnect if critical flags changed
        if oldSettings.localNetworkSharing != newSettings.localNetworkSharing
            || oldSettings.includeAllNetworks != newSettings.includeAllNetworks
        {
            return .hardReconnect
        }

        // If all non-location fields are equal, retry only when location settings change
        if oldSettings.withoutLocation == newSettings.withoutLocation
            && (oldSettings.relayConstraints.entryLocations.value?.locations
                == newSettings.relayConstraints.entryLocations.value?.locations)
            && (oldSettings.relayConstraints.exitLocations.value?.locations
                == newSettings.relayConstraints.exitLocations.value?.locations)
        {
            return .none
        }

        // If settings did not change, reconnect to the current relay
        guard oldSettings != newSettings else {
            return .currentRelayReconnect
        }

        return .newRelayReconnect
    }
}

/// This enum represents reconnection strategies.
/// > Warning: `hardReconnect` will disconnect and reconnect which
/// > potentially leads to traffic leaking outside the tunnel.
public enum TunnelSettingsReconnectionStrategy {
    case currentRelayReconnect
    case newRelayReconnect
    case hardReconnect
    case none
}

extension LatestTunnelSettings {
    var withoutLocation: LatestTunnelSettings {
        var copy = self
        copy.relayConstraints.exitLocations = .any
        copy.relayConstraints.entryLocations = .any
        return copy
    }
}
