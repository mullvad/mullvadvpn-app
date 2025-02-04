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
    let obfuscation: ObfuscatorPortSelection
    let constraints: RelayConstraints
    let connectionAttemptCount: UInt
    let daitaSettings: DAITASettings

    func pick() throws -> SelectedRelays {
        do {
            return try pick(from: obfuscation.exitRelays)
        } catch let error as NoRelaysSatisfyingConstraintsError where error.reason == .noDaitaRelaysFound {
            // If DAITA is on, Direct only is off and obfuscation is on, and no supported relays are found, we should see if
            // the obfuscated subset of exit relays is the cause of this. We can do this by checking if relay selection would
            // have been successful with all relays available. If that's the case, throw error and point to obfuscation.
            do {
                _ = try pick(from: obfuscation.unfilteredRelays)
                throw NoRelaysSatisfyingConstraintsError(.noObfuscatedRelaysFound)
            } catch let error as NoRelaysSatisfyingConstraintsError where error.reason == .noDaitaRelaysFound {
                // If DAITA is on, Direct only is off and obfuscation has been ruled out, and no supported relays are found,
                // we should try to find the nearest available relay that supports DAITA and use it as entry in a multihop selection.
                if daitaSettings.isAutomaticRouting {
                    return try MultihopPicker(
                        obfuscation: obfuscation,
                        constraints: constraints,
                        connectionAttemptCount: connectionAttemptCount,
                        daitaSettings: daitaSettings
                    ).pick()
                } else {
                    throw error
                }
            }
        }
    }

    private func pick(from exitRelays: REST.ServerRelaysResponse) throws -> SelectedRelays {
        let exitCandidates = try RelaySelector.WireGuard.findCandidates(
            by: constraints.exitLocations,
            in: exitRelays,
            filterConstraint: constraints.filter,
            daitaEnabled: daitaSettings.daitaState.isEnabled
        )

        let match = try findBestMatch(from: exitCandidates, useObfuscatedPortIfAvailable: true)
        return SelectedRelays(entry: nil, exit: match, retryAttempt: connectionAttemptCount)
    }
}
