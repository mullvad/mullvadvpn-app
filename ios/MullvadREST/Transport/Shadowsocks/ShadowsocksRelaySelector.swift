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
        with constraints: RelayConstraints,
        multihopState: MultihopState
    ) throws -> REST.BridgeRelay?

    func getBridges() throws -> REST.ServerShadowsocks?
}

final public class ShadowsocksRelaySelector: ShadowsocksRelaySelectorProtocol {
    let relayCache: RelayCacheProtocol

    public init(
        relayCache: RelayCacheProtocol
    ) {
        self.relayCache = relayCache
    }

    public func selectRelay(
        with constraints: RelayConstraints,
        multihopState: MultihopState
    ) throws -> REST.BridgeRelay? {
        let cachedRelays = try relayCache.read().relays

        // TODO: Will be removed in an upcoming PR when the feature is more complete.
        #if DEBUG
        var multihopState = multihopState
        multihopState = .off
        #endif

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
