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
            connectionAttemptCount: connectionAttemptCount,
            obfuscationBypass: IdentityObfuscationProvider()
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
            connectionAttemptCount: 0,
            obfuscationBypass: IdentityObfuscationProvider()
        ).obfuscate()

        let findCandidates:
            (REST.ServerRelaysResponse, Bool, RelayConstraint) throws
                -> [RelayWithLocation<REST.ServerRelay>] = { relays, daitaEnabled, filter in
                    try RelaySelector.WireGuard.findCandidates(
                        by: .any,
                        in: relays,
                        filterConstraint: filter,
                        daitaEnabled: daitaEnabled
                    )
                }

        return if tunnelSettings.daita.isAutomaticRouting {
            // When "Direct only" is not enabled the user will pick from the exit relays and
            // is then multihopped to a compatible server if necessary. We need to apply the
            // obfuscated relays to exit selection too so that the user doesn't pick
            // anything that isn't available for the entry server IF multihop DOESN'T kick in.
            RelayCandidates(
                entryRelays: try findCandidates(
                    obfuscation.obfuscatedRelays,
                    tunnelSettings.daita.daitaState.isEnabled,
                    tunnelSettings.relayConstraints.entryFilter
                ),
                exitRelays: try findCandidates(
                    // If multihop is explicitly enabled as well, any exit should be viable.
                    tunnelSettings.tunnelMultihopState.isEnabled ? obfuscation.allRelays : obfuscation.obfuscatedRelays,
                    false,
                    tunnelSettings.relayConstraints.exitFilter
                )
            )
        } else if tunnelSettings.tunnelMultihopState.isEnabled {
            // Any exit is viable due to multihop. DAITA and obfuscation is applied on
            // the entry only.
            RelayCandidates(
                entryRelays: try findCandidates(
                    obfuscation.obfuscatedRelays,
                    tunnelSettings.daita.daitaState.isEnabled,
                    tunnelSettings.relayConstraints.entryFilter
                ),
                exitRelays: try findCandidates(
                    obfuscation.allRelays,
                    false,
                    tunnelSettings.relayConstraints.exitFilter
                )
            )
        } else {
            // Singlehop. Always apply DAITA and obfuscation.
            RelayCandidates(
                entryRelays: nil,
                exitRelays: try findCandidates(
                    obfuscation.obfuscatedRelays,
                    tunnelSettings.daita.daitaState.isEnabled,
                    tunnelSettings.relayConstraints.exitFilter
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
                            return (minPort...maxPort).contains(port)
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
