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
        do {
            return try pick(from: obfuscation.allRelays)
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

    private func pick(from exitRelays: REST.ServerRelaysResponse) throws -> SelectedRelays {
        let exitCandidates = try RelaySelector.WireGuard.findCandidates(
            by: tunnelSettings.relayConstraints.exitLocations,
            in: exitRelays,
            filterConstraint: tunnelSettings.relayConstraints.filter,
            daitaEnabled: tunnelSettings.daita.daitaState.isEnabled,
            relaysForFilteringObfuscation: obfuscation.obfuscatedRelays
        )

        let match = try findBestMatch(from: exitCandidates, useObfuscatedPortIfAvailable: true)
        return SelectedRelays(entry: nil, exit: match, retryAttempt: connectionAttemptCount)
    }
}
