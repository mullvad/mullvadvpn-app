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

final class RelaySelectorWrapper: RelaySelectorProtocol {
    let relayCache: RelayCacheProtocol
    let multihopUpdater: MultihopUpdater
    private var multihopState: MultihopState = .off
    private var observer: MultihopObserverBlock!

    deinit {
        self.multihopUpdater.removeObserver(observer)
    }

    public init(
        relayCache: RelayCacheProtocol,
        multihopUpdater: MultihopUpdater
    ) {
        self.relayCache = relayCache
        self.multihopUpdater = multihopUpdater
        self.addObserver()
    }

    private func addObserver() {
        self.observer = MultihopObserverBlock(didUpdateMultihop: { [weak self] _, multihopState in
            self?.multihopState = multihopState
        })

        multihopUpdater.addObserver(observer)
    }

    func selectRelay(
        with constraints: RelayConstraints,
        connectionAttemptFailureCount: UInt
    ) throws -> SelectedRelay {
        switch multihopState {
        case .off, .on:
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
        }
    }
}
