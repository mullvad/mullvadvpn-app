//
//  SinglehopPicker.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-12-11.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import MullvadTypes

struct SinglehopPicker: RelayPicking {
    let obfuscation: RelayObfuscation
    let tunnelSettings: LatestTunnelSettings
    let connectionAttemptCount: UInt

    func shouldTriggerMultihop(reason: NoRelaysSatisfyingConstraintsReason) -> Bool {
        guard tunnelSettings.multihopEverwhere else { return false }

        return switch reason {
        case .noDaitaRelaysFound, .noObfuscatedRelaysFound:
            true
        case .filterConstraintNotMatching, .invalidPort, .entryEqualsExit,
                .multihopInvalidFlow, .noActiveRelaysFound, .relayConstraintNotMatching:
            false
        }
    }

    func pick() throws -> SelectedRelays {
        do {
            return try pick(from: obfuscation.allRelays)
        } catch let error as NoRelaysSatisfyingConstraintsError where shouldTriggerMultihop(reason: error.reason) {
            return try MultihopPicker(
                obfuscation: obfuscation,
                tunnelSettings: tunnelSettings,
                connectionAttemptCount: connectionAttemptCount
            ).pick()
        }
    }

    private func pick(from obfuscatedRelays: REST.ServerRelaysResponse) throws -> SelectedRelays {
        let constraints = tunnelSettings.relayConstraints
        let daitaSettings = tunnelSettings.daita

        let supportedObfuscation = RelayObfuscator(
            relays: obfuscation.allRelays,
            tunnelSettings: tunnelSettings,
            connectionAttemptCount: connectionAttemptCount
        ).obfuscate()

        let exitCandidates = try RelaySelector.WireGuard.findCandidates(
            by: tunnelSettings.relayConstraints.exitLocations,
            in: supportedObfuscation.allRelays,
            filterConstraint: constraints.filter,
            daitaEnabled: daitaSettings.daitaState.isEnabled,
            obfuscation: supportedObfuscation
        )

        let match = try findBestMatch(from: exitCandidates, useObfuscatedPortIfAvailable: true)
        return SelectedRelays(
            entry: nil,
            exit: match,
            retryAttempt: connectionAttemptCount,
            obfuscation: supportedObfuscation.method
        )
    }
}
