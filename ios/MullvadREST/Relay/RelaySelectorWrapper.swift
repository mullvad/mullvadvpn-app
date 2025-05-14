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
        try validateWireguardPort(tunnelSettings, relays: relays)

        let obfuscation = try prepareObfuscation(
            for: tunnelSettings,
            connectionAttemptCount: connectionAttemptCount,
            relays: relays
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

    public func findCandidates(tunnelSettings: LatestTunnelSettings) throws -> RelayCandidates {
        let relays = try relayCache.read().relays
        let obfuscation = try prepareObfuscation(
            for: tunnelSettings,
            connectionAttemptCount: 0,
            relays: relays
        )

        let findCandidates: (REST.ServerRelaysResponse, Bool) throws
            -> [RelayWithLocation<REST.ServerRelay>] = { relays, daitaEnabled in
                try RelaySelector.WireGuard.findCandidates(
                    by: .any,
                    in: relays,
                    filterConstraint: tunnelSettings.relayConstraints.filter,
                    daitaEnabled: daitaEnabled
                )
            }

        if tunnelSettings.daita.isAutomaticRouting || tunnelSettings.tunnelMultihopState.isEnabled {
            let entryCandidates = try findCandidates(
                tunnelSettings.tunnelMultihopState.isEnabled ? obfuscation.entryRelays : obfuscation.exitRelays,
                tunnelSettings.daita.daitaState.isEnabled
            )
            let exitCandidates = try findCandidates(obfuscation.exitRelays, false)
            return RelayCandidates(entryRelays: entryCandidates, exitRelays: exitCandidates)
        } else {
            let exitCandidates = try findCandidates(obfuscation.exitRelays, tunnelSettings.daita.daitaState.isEnabled)
            return RelayCandidates(entryRelays: nil, exitRelays: exitCandidates)
        }
    }

    private func prepareObfuscation(
        for tunnelSettings: LatestTunnelSettings,
        connectionAttemptCount: UInt,
        relays: REST.ServerRelaysResponse
    ) throws -> ObfuscatorPortSelection {
        return try ObfuscatorPortSelector(relays: relays).obfuscate(
            tunnelSettings: tunnelSettings,
            connectionAttemptCount: connectionAttemptCount
        )
    }

    private func validateWireguardPort(
        _ tunnelSettings: LatestTunnelSettings,
        relays: REST.ServerRelaysResponse
    ) throws {
        switch tunnelSettings.wireGuardObfuscation.state {
        case .automatic, .off:
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
        case .on, .udpOverTcp, .quic, .shadowsocks:
            break
        }
    }
}
