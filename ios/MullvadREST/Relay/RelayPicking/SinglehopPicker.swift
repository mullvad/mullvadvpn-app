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

struct SinglehopPicker: RelayPicking {
    let logger = Logger(label: "SinglehopPicker")
    let obfuscation: RelayObfuscation
    let tunnelSettings: LatestTunnelSettings
    let connectionAttemptCount: UInt

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

    func pick() throws -> SelectedRelays {
        do {
            let obfuscationBypass = UnsupportedObfuscationProvider(
                relayConstraint: tunnelSettings.relayConstraints.exitLocations,
                relays: obfuscation.obfuscatedRelays,
                filterConstraint: tunnelSettings.relayConstraints.exitFilter,
                daitaEnabled: tunnelSettings.daita.daitaState.isEnabled
            )

            let supportedObfuscation = try RelayObfuscator(
                relays: obfuscation.allRelays,
                tunnelSettings: tunnelSettings,
                connectionAttemptCount: connectionAttemptCount,
                obfuscationBypass: obfuscationBypass
            ).obfuscate()

            return try SinglehopPicker(
                obfuscation: supportedObfuscation, tunnelSettings: tunnelSettings,
                connectionAttemptCount: connectionAttemptCount
            ).pick(from: supportedObfuscation.obfuscatedRelays)
        } catch let error as NoRelaysSatisfyingConstraintsError where shouldTriggerMultihop(reason: error.reason) {
            return try MultihopPicker(
                obfuscation: obfuscation,
                tunnelSettings: tunnelSettings,
                connectionAttemptCount: connectionAttemptCount
            ).pick()
        }
    }

    private func pick(from relaysResponse: REST.ServerRelaysResponse) throws -> SelectedRelays {
        let exitCandidates = try RelaySelector.WireGuard.findCandidates(
            by: tunnelSettings.relayConstraints.exitLocations,
            in: relaysResponse,
            filterConstraint: tunnelSettings.relayConstraints.exitFilter,
            daitaEnabled: tunnelSettings.daita.daitaState.isEnabled
        )

        let match = try findBestMatch(from: exitCandidates, applyObfuscation: true)

        return SelectedRelays(
            entry: nil,
            exit: match,
            retryAttempt: connectionAttemptCount
        )
    }
}
