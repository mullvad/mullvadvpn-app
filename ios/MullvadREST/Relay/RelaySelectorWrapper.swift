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

    public init(relayCache: RelayCacheProtocol) {
        self.relayCache = relayCache
    }

    public func selectRelays(
        tunnelSettings: LatestTunnelSettings,
        connectionAttemptCount: UInt
    ) throws -> SelectedRelays {
        let relays = try relayCache.read().relays

        switch tunnelSettings.tunnelMultihopState {
        case .off:
            return try SinglehopPicker(
                constraints: tunnelSettings.relayConstraints,
                daitaSettings: tunnelSettings.daita,
                relays: relays,
                connectionAttemptCount: connectionAttemptCount
            ).pick()
        case .on:
            return try MultihopPicker(
                constraints: tunnelSettings.relayConstraints,
                daitaSettings: tunnelSettings.daita,
                relays: relays,
                connectionAttemptCount: connectionAttemptCount
            ).pick()
        }
    }
}
