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

    func pick() throws -> SelectedRelays {
        // Guarantee that the chosen relay supports selected obfuscation
        let obfuscationBypass = UnsupportedObfuscationProvider(
            relayConstraint: tunnelSettings.relayConstraints.exitLocations,
            relays: obfuscation.obfuscatedRelays,
            filterConstraint: tunnelSettings.relayConstraints.exitFilter,
            daitaEnabled: tunnelSettings.daita.daitaState.isEnabled
        )

        let supportedObfuscation = RelayObfuscator(
            relays: obfuscation.allRelays,
            tunnelSettings: tunnelSettings,
            connectionAttemptCount: connectionAttemptCount,
            obfuscationBypass: obfuscationBypass
        ).obfuscate()

        // Create a new picker so that it can use the new obfuscation object.
        let picker = SinglehopPicker(
            obfuscation: supportedObfuscation,
            tunnelSettings: tunnelSettings,
            connectionAttemptCount: connectionAttemptCount
        )

        do {
            return try picker.pickRelays()
        } catch let error as NoRelaysSatisfyingConstraintsError where error.reason == .noDaitaRelaysFound {
            // If DAITA is on, Direct only is off and obfuscation has been ruled out, and no supported relays are found,
            // we should try to find the nearest available relay that supports DAITA and use it as entry in a multihop selection.
            if tunnelSettings.daita.isAutomaticRouting {
                return try MultihopPicker(
                    obfuscation: obfuscation,
                    tunnelSettings: tunnelSettings,
                    connectionAttemptCount: connectionAttemptCount
                ).pick()
            } else {
                throw error
            }
        }
    }

    private func pickRelays() throws -> SelectedRelays {
        let exitCandidates = try RelaySelector.WireGuard.findCandidates(
            by: tunnelSettings.relayConstraints.exitLocations,
            in: obfuscation.obfuscatedRelays,
            filterConstraint: tunnelSettings.relayConstraints.exitFilter,
            daitaEnabled: tunnelSettings.daita.daitaState.isEnabled
        )

        let match = try findBestMatch(from: exitCandidates, applyObfuscatedIps: true)

        return SelectedRelays(
            entry: nil,
            exit: match,
            retryAttempt: connectionAttemptCount,
            obfuscation: obfuscation.method
        )
    }
}
