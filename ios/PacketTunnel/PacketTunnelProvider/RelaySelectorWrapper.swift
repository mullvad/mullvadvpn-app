//
//  RelaySelectorWrapper.swift
//  PacketTunnel
//
//  Created by pronebird on 08/08/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadTypes
import PacketTunnelCore

struct MultihopNotImplementedError: LocalizedError {
    public var errorDescription: String? {
        "Picking relays for Multihop is not implemented yet."
    }
}

struct RelaySelectorWrapper: RelaySelectorProtocol {
    let relayCache: RelayCacheProtocol
    let settingsReader: SettingsReaderProtocol

    func selectRelay(
        with constraints: RelayConstraints,
        connectionAttemptFailureCount: UInt
    ) throws -> SelectedRelay {
        switch try settingsReader.read().multihopState {
        case .off:
            let selectorResult = try RelaySelector.WireGuard.evaluateForSingleHop(
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
