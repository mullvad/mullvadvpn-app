//
//  RelaySelectorWrapper.swift
//  PacketTunnel
//
//  Created by pronebird on 08/08/2023.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
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

        return switch tunnelSettings.tunnelMultihopState {
        case .never, .whenNeeded:
            try SinglehopPicker(
                relays: relays,
                tunnelSettings: tunnelSettings,
                connectionAttemptCount: connectionAttemptCount
            ).pick()
        case .always:
            try MultihopPicker(
                relays: relays,
                tunnelSettings: tunnelSettings,
                connectionAttemptCount: connectionAttemptCount
            ).pick()
        }
    }

    /// This function is expected to be used by the UI to list all available servers, even the inactive ones.
    /// For the purposes of creating a relay connection, we should use `selectRelays` instead.
    public func findCandidates(tunnelSettings: LatestTunnelSettings) throws -> RelayCandidates {
        let relays = try relayCache.read().relays

        let obfuscation = try RelayObfuscator(
            relays: relays,
            tunnelSettings: tunnelSettings,
            connectionAttemptCount: 0,
            obfuscationBypass: IdentityObfuscationProvider()
        ).obfuscate()

        let findCandidates:
            (REST.ServerRelaysResponse, Bool, RelayConstraint, RelayObfuscation?) throws
                -> [RelayWithLocation<REST.ServerRelay>] = { relays, daitaEnabled, filter, obfuscation in
                    try RelaySelector.WireGuard.findCandidates(
                        by: .any,
                        in: relays,
                        filterConstraint: filter,
                        daitaEnabled: daitaEnabled,
                        obfuscation: obfuscation,
                        includeInactive: true
                    )
                }

        return if tunnelSettings.automaticMultihopIsEnabled {
            RelayCandidates(
                entryRelays: try findCandidates(
                    relays,
                    tunnelSettings.daita.isEnabled,
                    .any,
                    obfuscation
                ),
                exitRelays: try findCandidates(
                    relays,
                    false,
                    tunnelSettings.relayConstraints.exitFilter,
                    nil
                )
            )
        } else if tunnelSettings.tunnelMultihopState.isAlways {
            RelayCandidates(
                entryRelays: try findCandidates(
                    relays,
                    tunnelSettings.daita.isEnabled,
                    tunnelSettings.relayConstraints.entryFilter,
                    obfuscation
                ),
                exitRelays: try findCandidates(
                    relays,
                    false,
                    tunnelSettings.relayConstraints.exitFilter,
                    nil
                )
            )
        } else {
            RelayCandidates(
                entryRelays: nil,
                exitRelays: try findCandidates(
                    relays,
                    tunnelSettings.daita.isEnabled,
                    tunnelSettings.relayConstraints.exitFilter,
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
