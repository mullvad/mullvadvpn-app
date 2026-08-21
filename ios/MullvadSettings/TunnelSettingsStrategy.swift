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
        switch (oldSettings, newSettings) {
        case let (old, new) where old.includeAllNetworks != new.includeAllNetworks:
            return .hardReconnect
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
}
