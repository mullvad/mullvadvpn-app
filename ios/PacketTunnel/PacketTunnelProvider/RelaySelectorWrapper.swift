//
//  RelaySelectorWrapper.swift
//  PacketTunnel
//
//  Created by pronebird on 08/08/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadSettings
import MullvadTypes
import PacketTunnelCore

struct MultihopNotImplementedError: LocalizedError {
    public var errorDescription: String? {
        "Picking relays for Multihop is not implemented yet."
    }
}

final class RelaySelectorWrapper: RelaySelectorProtocol {
    let relayCache: RelayCacheProtocol
    let multihopStateUpdater: MultihopStateUpdater
    private var multihopState: MultihopState = .off

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

    func selectRelay(
        with constraints: RelayConstraints,
        connectionAttemptFailureCount: UInt
    ) throws -> SelectedRelay {
        switch multihopState {
        case .off:
            let selectorResult = try RelaySelector.WireGuard.evaluate(
                by: constraints,
                in: relayCache.read().relays,
                numberOfFailedAttempts: connectionAttemptFailureCount
            )

            return SelectedRelay(
                endpoint: selectorResult.endpoint,
                hostname: selectorResult.relay.hostname,
                location: selectorResult.location,
                retryAttempts: connectionAttemptFailureCount
            )

        case .on:
            throw MultihopNotImplementedError()
        }
    }
}
