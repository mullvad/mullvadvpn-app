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
import MullvadSettings
import MullvadTypes

public protocol ShadowsocksRelaySelectorProtocol: Sendable {
    func selectBridge(with settings: LatestTunnelSettings) throws -> REST.BridgeRelay?
    func getBridgeConfig() throws -> REST.ServerShadowsocks?
}

final public class ShadowsocksRelaySelector: ShadowsocksRelaySelectorProtocol {
    let relayCache: RelayCacheProtocol

    public init(
        relayCache: RelayCacheProtocol
    ) {
        self.relayCache = relayCache
    }

    public func selectBridge(with settings: LatestTunnelSettings) throws -> REST.BridgeRelay? {
        let cachedRelays = try relayCache.read().relays

        let locationConstraint =
            switch settings.tunnelMultihopState {
            case .always: settings.relayConstraints.entryLocations
            case .never, .whenNeeded: settings.relayConstraints.exitLocations
            }

        return RelaySelector.Shadowsocks.closestBridge(
            location: locationConstraint,
            in: cachedRelays
        )
    }

    public func getBridgeConfig() throws -> REST.ServerShadowsocks? {
        let cachedRelays = try relayCache.read()
        return RelaySelector.Shadowsocks.randomBridgeConfig(from: cachedRelays.relays)
    }
}
