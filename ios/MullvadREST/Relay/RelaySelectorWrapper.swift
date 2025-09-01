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

        let obfuscation = try RelayObfuscator(relays: relays).obfuscate(
            tunnelSettings: tunnelSettings,
            connectionAttemptCount: connectionAttemptCount
        )

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

        let obfuscation = try RelayObfuscator(relays: relays).obfuscate(
            tunnelSettings: tunnelSettings,
            connectionAttemptCount: 0
        )

        let findCandidates: (REST.ServerRelaysResponse, Bool, RelayObfuscation?) throws
            -> [RelayWithLocation<REST.ServerRelay>] = { relays, daitaEnabled, obfuscation in
                try RelaySelector.WireGuard.findCandidates(
                    by: .any,
                    in: relays,
                    filterConstraint: tunnelSettings.relayConstraints.filter,
                    daitaEnabled: daitaEnabled,
                    relaysForFilteringObfuscation: obfuscation?.obfuscatedRelays
                )
            }

        if tunnelSettings.daita.isAutomaticRouting || tunnelSettings.tunnelMultihopState.isEnabled {
            let entryCandidates = try findCandidates(
                obfuscation.obfuscatedRelays,
                tunnelSettings.daita.daitaState.isEnabled,
                obfuscation
            )
            let exitCandidates = try findCandidates(obfuscation.allRelays, false, nil)
            return RelayCandidates(entryRelays: entryCandidates, exitRelays: exitCandidates)
        } else {
            let exitCandidates = try findCandidates(
                obfuscation.obfuscatedRelays,
                tunnelSettings.daita.daitaState.isEnabled,
                nil
            )
            return RelayCandidates(entryRelays: nil, exitRelays: exitCandidates)
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
