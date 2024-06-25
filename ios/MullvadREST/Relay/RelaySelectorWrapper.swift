//
//  RelaySelectorWrapper.swift
//  PacketTunnel
//
//  Created by pronebird on 08/08/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import MullvadTypes

public final class RelaySelectorWrapper: RelaySelectorProtocol {
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

    public func selectRelays(
        with constraints: RelayConstraints,
        connectionAttemptCount: UInt
    ) throws -> SelectedRelays {
        let relays = try relayCache.read().relays

        switch multihopState {
        case .off:
            return try SinglehopPicker(
                constraints: constraints,
                relays: relays,
                connectionAttemptCount: connectionAttemptCount
            ).pick()
        case .on:
            return try MultihopPicker(
                constraints: constraints,
                relays: relays,
                connectionAttemptCount: connectionAttemptCount
            ).pick()
        }
    }

    private func addObserver() {
        self.observer = MultihopObserverBlock(didUpdateMultihop: { [weak self] _, multihopState in
            self?.multihopState = multihopState
        })

        multihopUpdater.addObserver(observer)
    }
}
