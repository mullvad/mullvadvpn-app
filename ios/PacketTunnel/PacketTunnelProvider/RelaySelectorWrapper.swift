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

struct RelaySelectorWrapper: RelaySelectorProtocol {
    let relayCache: RelayCacheProtocol

    func selectRelay(
        with constraints: RelayConstraints,
        connectionAttemptFailureCount: UInt
    ) throws -> SelectedRelay {
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
