//
//  RelaySelectorWrapper.swift
//  PacketTunnel
//
//  Created by pronebird on 08/08/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes
import PacketTunnelCore
import RelayCache
import RelaySelector

struct RelaySelectorWrapper: RelaySelectorProtocol {
    let relayCache: RelayCache

    func selectRelay(
        with constraints: RelayConstraints,
        connectionAttemptFailureCount: UInt
    ) throws -> SelectedRelay {
        let selectorResult = try RelaySelector.evaluate(
            relays: relayCache.read().relays,
            constraints: constraints,
            numberOfFailedAttempts: connectionAttemptFailureCount
        )

        return SelectedRelay(
            endpoint: selectorResult.endpoint,
            hostname: selectorResult.relay.hostname,
            location: selectorResult.location
        )
    }
}
