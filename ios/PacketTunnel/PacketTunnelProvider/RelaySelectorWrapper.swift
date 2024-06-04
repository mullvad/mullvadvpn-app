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
    let multihopUpdater: MultihopUpdater
    private var multihopState: MultihopState = .off
    private var observer: MultihopObserverBlock!

    deinit {
        self.multihopUpdater.removeObserver(observer)
    }

    public init(
        relayCache: RelayCacheProtocol,
        multihopUpdater: MultihopUpdater,
        multihopState: MultihopState
    ) {
        self.relayCache = relayCache
        self.multihopState = multihopState
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
