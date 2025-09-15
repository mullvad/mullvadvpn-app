//
//  MultihopPicker.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-12-11.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import MullvadTypes

struct MultihopPicker: RelayPicking {
    let obfuscation: RelayObfuscation
    let tunnelSettings: LatestTunnelSettings
    let connectionAttemptCount: UInt

    func pick() throws -> SelectedRelays {
        let constraints = tunnelSettings.relayConstraints
        let daitaSettings = tunnelSettings.daita

        // Guarantee that the entry relay supports selected obfuscation
        let obfuscationBypass = UnsupportedObfuscationProvider(
            relayConstraint: constraints.entryLocations,
            relays: obfuscation.obfuscatedRelays,
            filterConstraint: constraints.filter,
            daitaEnabled: daitaSettings.daitaState.isEnabled
        )

        let supportedObfuscation = try RelayObfuscator(
            relays: obfuscation.allRelays,
            tunnelSettings: tunnelSettings,
            connectionAttemptCount: connectionAttemptCount,
            obfuscationBypass: obfuscationBypass
        ).obfuscate()

        let entryCandidates = try RelaySelector.WireGuard.findCandidates(
            by: daitaSettings.isAutomaticRouting ? .any : constraints.entryLocations,
            in: supportedObfuscation.obfuscatedRelays,
            filterConstraint: constraints.filter,
            daitaEnabled: daitaSettings.daitaState.isEnabled
        )

        let exitCandidates = try RelaySelector.WireGuard.findCandidates(
            by: constraints.exitLocations,
            in: supportedObfuscation.allRelays,
            filterConstraint: constraints.filter,
            daitaEnabled: false
        )

        let picker = MultihopPicker(
            obfuscation: supportedObfuscation,
            tunnelSettings: tunnelSettings,
            connectionAttemptCount: connectionAttemptCount
        )
        /*
         Relay selection is prioritised in the following order:
         1. Both entry and exit constraints match only a single relay. Both relays are selected.
         2. Entry constraint matches only a single relay and the other multiple relays. The single relay
            is selected and excluded from the list of multiple relays.
         3. Exit constraint matches multiple relays and the other a single relay. The single relay
            is selected and excluded from the list of multiple relays.
         4. Both entry and exit constraints match multiple relays. Exit relay is picked first and then
            excluded from the list of entry relays.
         */
        let decisionFlow = OneToOne(
            next: OneToMany(
                next: ManyToOne(
                    next: ManyToMany(
                        next: nil,
                        relayPicker: picker
                    ),
                    relayPicker: picker
                ),
                relayPicker: picker
            ),
            relayPicker: picker
        )

        return try decisionFlow.pick(
            entryCandidates: entryCandidates,
            exitCandidates: exitCandidates,
            daitaAutomaticRouting: daitaSettings.isAutomaticRouting
        )
    }

    func exclude(
        relay: SelectedRelay,
        from candidates: [RelayWithLocation<REST.ServerRelay>],
        closeTo location: Location? = nil,
        useObfuscatedPortIfAvailable: Bool
    ) throws -> SelectedRelay {
        let filteredCandidates = candidates.filter { relayWithLocation in
            relayWithLocation.relay.hostname != relay.hostname
        }

        return try findBestMatch(
            from: filteredCandidates,
            closeTo: location,
            useObfuscatedPortIfAvailable: useObfuscatedPortIfAvailable
        )
    }
}
