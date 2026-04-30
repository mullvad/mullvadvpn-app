//
//  SinglehopPicker.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-12-11.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadLogging
import MullvadSettings
import MullvadTypes

public struct SinglehopPicker: RelayPicking {
    public let logger = Logger(label: "SinglehopPicker")
    public let relays: REST.ServerRelaysResponse
    public let tunnelSettings: LatestTunnelSettings
    public let connectionAttemptCount: UInt

    public init(relays: REST.ServerRelaysResponse, tunnelSettings: LatestTunnelSettings, connectionAttemptCount: UInt) {
        self.relays = relays
        self.tunnelSettings = tunnelSettings
        self.connectionAttemptCount = connectionAttemptCount
    }

    public func pick() throws -> SelectedRelays {
        do {
            let obfuscationBypass = UnsupportedObfuscationProvider(
                relayConstraint: tunnelSettings.relayConstraints.exitLocations,
                relays: relays,
                filterConstraint: tunnelSettings.relayConstraints.exitFilter,
                daitaEnabled: tunnelSettings.daita.isEnabled
            )

            let obfuscation = try RelayObfuscator(
                relays: relays,
                tunnelSettings: tunnelSettings,
                connectionAttemptCount: connectionAttemptCount,
                obfuscationBypass: obfuscationBypass
            ).obfuscate()

            let exitCandidates = try RelaySelector.WireGuard.findCandidates(
                by: tunnelSettings.relayConstraints.exitLocations,
                in: relays,
                filterConstraint: tunnelSettings.relayConstraints.exitFilter,
                daitaEnabled: tunnelSettings.daita.isEnabled,
                obfuscation: obfuscation
            )

            let match = try findBestMatch(from: exitCandidates, obfuscation: obfuscation)

            return SelectedRelays(
                entry: nil,
                exit: match,
                retryAttempt: connectionAttemptCount
            )
        } catch let error as NoRelaysSatisfyingConstraintsError where shouldTriggerMultihop(reason: error.reason) {
            return try MultihopPicker(
                relays: relays,
                tunnelSettings: tunnelSettings,
                connectionAttemptCount: connectionAttemptCount
            ).pick()
        }
    }

    private func shouldTriggerMultihop(reason: NoRelaysSatisfyingConstraintsReason) -> Bool {
        guard tunnelSettings.automaticMultihopIsEnabled else { return false }

        return switch reason {
        case .noDaitaRelaysFound, .noObfuscatedRelaysFound, .noIPv6RelayFound:
            true
        case .filterConstraintNotMatching, .invalidPort, .entryEqualsExit,
            .multihopInvalidFlow, .noActiveRelaysFound, .relayConstraintNotMatching, .invalidObfuscationPort:
            false
        }
    }
}
