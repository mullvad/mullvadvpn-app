//
//  ShadowsocksRelaySelector.swift
//  MullvadREST
//
//  Created by Mojgan on 2024-05-23.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings
import MullvadTypes

public protocol ShadowsocksRelaySelectorProtocol {
    func selectRelay(
        with constraints: RelayConstraints
    ) throws -> REST.BridgeRelay?

    func getBridges() throws -> REST.ServerShadowsocks?
}

final public class ShadowsocksRelaySelector: ShadowsocksRelaySelectorProtocol {
    let relayCache: RelayCacheProtocol
    let multihopStateUpdater: MultihopStateUpdater
    private var multihopState: MultihopState

    public init(
        relayCache: RelayCacheProtocol,
        multihopState: MultihopState,
        multihopStateUpdater: MultihopStateUpdater
    ) {
        self.relayCache = relayCache
        self.multihopState = multihopState
        self.multihopStateUpdater = multihopStateUpdater

        multihopStateUpdater.onNewState = { [weak self] newState in
            self?.multihopState = newState
        }
    }

    public func selectRelay(
        with constraints: RelayConstraints
    ) throws -> REST.BridgeRelay? {
        let cachedRelays = try relayCache.read().relays
        switch multihopState {
        case .on:
            return RelaySelector.Shadowsocks.closestRelay(
                location: constraints.entryLocations,
                port: constraints.port,
                filter: constraints.filter,
                in: cachedRelays
            )
        case .off:
            return RelaySelector.Shadowsocks.closestRelay(
                location: constraints.exitLocations,
                port: constraints.port,
                filter: constraints.filter,
                in: cachedRelays
            )
        }
    }

    public func getBridges() throws -> REST.ServerShadowsocks? {
        let cachedRelays = try relayCache.read()
        return RelaySelector.Shadowsocks.tcpBridge(from: cachedRelays.relays)
    }
}
