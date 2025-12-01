//
//  ShadowsocksRelaySelector.swift
//  MullvadREST
//
//  Created by Mojgan on 2024-05-23.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings
import MullvadTypes

public protocol ShadowsocksRelaySelectorProtocol: Sendable {
    func selectRelay(with settings: LatestTunnelSettings) throws -> REST.BridgeRelay?

    func getBridge() throws -> REST.ServerShadowsocks?
}

final public class ShadowsocksRelaySelector: ShadowsocksRelaySelectorProtocol {
    let relayCache: RelayCacheProtocol

    public init(
        relayCache: RelayCacheProtocol
    ) {
        self.relayCache = relayCache
    }

    public func selectRelay(with settings: LatestTunnelSettings) throws -> REST.BridgeRelay? {
        let cachedRelays = try relayCache.read().relays

        let locationConstraint =
            switch settings.tunnelMultihopState {
            case .on: settings.relayConstraints.entryLocations
            case .off: settings.relayConstraints.exitLocations
            }

        return RelaySelector.Shadowsocks.closestRelay(
            location: locationConstraint,
            port: settings.relayConstraints.port,
            filter: settings.relayConstraints.filter,
            in: cachedRelays
        )
    }

    public func getBridge() throws -> REST.ServerShadowsocks? {
        let cachedRelays = try relayCache.read()
        return RelaySelector.Shadowsocks.randomBridge(from: cachedRelays.relays)
    }
}
