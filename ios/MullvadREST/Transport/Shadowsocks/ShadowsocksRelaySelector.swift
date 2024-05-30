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
    let multihopUpdater: MultihopUpdater
    private var multihopState: MultihopState = .off

    public init(
        relayCache: RelayCacheProtocol,
        multihopUpdater: MultihopUpdater
    ) {
        self.relayCache = relayCache
        self.multihopUpdater = multihopUpdater

        multihopUpdater.addObserver(MultihopObserverBlock(didUpdateMultihop: { [weak self] _, multihopState in
            self?.multihopState = multihopState
        }))
    }

    public func selectRelay(
        with constraints: RelayConstraints
    ) throws -> REST.BridgeRelay? {
        let cachedRelays = try relayCache.read().relays

        let locationConstraint = switch multihopState {
        case .on: constraints.entryLocations
        case .off: constraints.exitLocations
        }

        return RelaySelector.Shadowsocks.closestRelay(
            location: locationConstraint,
            port: constraints.port,
            filter: constraints.filter,
            in: cachedRelays
        )
    }

    public func getBridges() throws -> REST.ServerShadowsocks? {
        let cachedRelays = try relayCache.read()
        return RelaySelector.Shadowsocks.tcpBridge(from: cachedRelays.relays)
    }
}
