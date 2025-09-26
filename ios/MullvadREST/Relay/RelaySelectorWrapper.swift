//
//  RelaySelectorWrapper.swift
//  PacketTunnel
//
//  Created by pronebird on 08/08/2023.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import MullvadTypes

public final class RelaySelectorWrapper: RelaySelectorProtocol, Sendable {
    public let relayCache: RelayCacheProtocol

    public init(relayCache: RelayCacheProtocol) {
        self.relayCache = relayCache
    }

    public func selectRelays(
        tunnelSettings: LatestTunnelSettings,
        connectionAttemptCount: UInt
    ) throws -> SelectedRelays {
        let relays = try relayCache.read().relays
        try validateWireguardCustomPort(tunnelSettings, relays: relays)

        // Filter for obfuscation
        let obfuscation = RelayObfuscator(
            relays: relays,
            tunnelSettings: tunnelSettings,
            connectionAttemptCount: connectionAttemptCount
        ).obfuscate()

        return switch tunnelSettings.tunnelMultihopState {
        case .off:
            try SinglehopPicker(
                obfuscation: obfuscation,
                tunnelSettings: tunnelSettings,
                connectionAttemptCount: connectionAttemptCount
            ).pick()
        case .on:
            try MultihopPicker(
                obfuscation: obfuscation,
                tunnelSettings: tunnelSettings,
                connectionAttemptCount: connectionAttemptCount
            ).pick()
        }
    }

    public func findCandidates(tunnelSettings: LatestTunnelSettings) throws -> RelayCandidates {
        let relays = try relayCache.read().relays

        let obfuscation = RelayObfuscator(
            relays: relays,
            tunnelSettings: tunnelSettings,
            connectionAttemptCount: 0
        ).obfuscate()

        let findCandidates: (REST.ServerRelaysResponse, Bool, RelayObfuscation?) throws
            -> [RelayWithLocation<REST.ServerRelay>] = { relays, daitaEnabled, obfuscation in
                try RelaySelector.WireGuard.findCandidates(
                    by: .any,
                    in: relays,
                    filterConstraint: tunnelSettings.relayConstraints.filter,
                    daitaEnabled: daitaEnabled,
                    obfuscation: obfuscation
                )
            }

        return if tunnelSettings.multihopEverwhere {
            RelayCandidates(
                entryRelays: try findCandidates(
                    obfuscation.obfuscatedRelays,
                    tunnelSettings.daita.daitaState.isEnabled,
                    obfuscation
                ),
                exitRelays: try findCandidates(
                    obfuscation.allRelays,
                    false,
                    nil
                )
            )
        } else if tunnelSettings.tunnelMultihopState.isEnabled {
            // Any exit is viable due to multihop. DAITA and obfuscation is applied on
            // the entry only.
            RelayCandidates(
                entryRelays: try findCandidates(
                    obfuscation.obfuscatedRelays,
                    tunnelSettings.daita.daitaState.isEnabled,
                    obfuscation
                ),
                exitRelays: try findCandidates(
                    obfuscation.allRelays,
                    false,
                    nil
                )
            )
        } else {
            // Singlehop. Always apply DAITA and obfuscation.
            RelayCandidates(
                entryRelays: nil,
                exitRelays: try findCandidates(
                    obfuscation.allRelays,
                    tunnelSettings.daita.daitaState.isEnabled,
                    obfuscation
                )
            )
        }
    }

    private func validateWireguardCustomPort(
        _ tunnelSettings: LatestTunnelSettings,
        relays: REST.ServerRelaysResponse
    ) throws {
        if [.automatic, .off].contains(tunnelSettings.wireGuardObfuscation.state) {
            if case let .only(port) = tunnelSettings.relayConstraints.port {
                let isPortWithinValidWireGuardRanges: Bool =
                    relays.wireguard.portRanges
                        .contains { range in
                            if let minPort = range.first, let maxPort = range.last {
                                return (minPort ... maxPort).contains(port)
                            }
                            return false
                        }
                guard isPortWithinValidWireGuardRanges else {
                    throw NoRelaysSatisfyingConstraintsError(.invalidPort)
                }
            }
        }
    }
}
