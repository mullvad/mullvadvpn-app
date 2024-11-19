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
        let obfuscation = try ObfuscatorPortSelector(
            relays: try relayCache.read().relays
        ).obfuscate(
            tunnelSettings: tunnelSettings,
            connectionAttemptCount: connectionAttemptCount
        )

        return switch tunnelSettings.tunnelMultihopState {
        case .off:
            try SinglehopPicker(
                obfuscation: obfuscation,
                constraints: tunnelSettings.relayConstraints,
                connectionAttemptCount: connectionAttemptCount,
                daitaSettings: tunnelSettings.daita
            ).pick()
        case .on:
            try MultihopPicker(
                obfuscation: obfuscation,
                constraints: tunnelSettings.relayConstraints,
                connectionAttemptCount: connectionAttemptCount,
                daitaSettings: tunnelSettings.daita
            ).pick()
        }
    }
}
