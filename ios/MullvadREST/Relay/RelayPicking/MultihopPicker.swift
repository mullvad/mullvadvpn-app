//
//  MultihopPicker.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-12-11.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadLogging
import MullvadSettings
import MullvadTypes

public struct MultihopPicker: RelayPicking {
    public let logger = Logger(label: "MultihopPicker")
    public let relays: REST.ServerRelaysResponse
    public let tunnelSettings: LatestTunnelSettings
    public let connectionAttemptCount: UInt

    public init(relays: REST.ServerRelaysResponse, tunnelSettings: LatestTunnelSettings, connectionAttemptCount: UInt) {
        self.relays = relays
        self.tunnelSettings = tunnelSettings
        self.connectionAttemptCount = connectionAttemptCount
    }

    public func pick() throws -> SelectedRelays {
        let constraints = tunnelSettings.relayConstraints
        let entryConstraints = tunnelSettings.automaticMultihopIsEnabled ? .any : constraints.entryLocations
        let entryFilter = tunnelSettings.automaticMultihopIsEnabled ? .any : constraints.entryFilter

        // Guarantee that the entry relay supports selected obfuscation
        let obfuscationBypass = UnsupportedObfuscationProvider(
            relayConstraint: entryConstraints,
            relays: relays,
            filterConstraint: entryFilter,
            daitaEnabled: tunnelSettings.daita.isEnabled
        )

        let obfuscation = try RelayObfuscator(
            relays: relays,
            tunnelSettings: tunnelSettings,
            connectionAttemptCount: connectionAttemptCount,
            obfuscationBypass: obfuscationBypass
        ).obfuscate()

        let entryCandidates = try RelaySelector.WireGuard.findCandidates(
            by: entryConstraints,
            in: relays,
            filterConstraint: entryFilter,
            daitaEnabled: tunnelSettings.daita.isEnabled,
            obfuscation: obfuscation
        )

        let exitCandidates = try RelaySelector.WireGuard.findCandidates(
            by: constraints.exitLocations,
            in: relays,
            filterConstraint: tunnelSettings.automaticMultihopIsEnabled ? .any : constraints.exitFilter,
            daitaEnabled: false,
            obfuscation: nil
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
                        relayPicker: self
                    ),
                    relayPicker: self
                ),
                relayPicker: self
            ),
            relayPicker: self
        )

        return try decisionFlow.pick(
            entryCandidates: entryCandidates,
            exitCandidates: exitCandidates,
            obfuscation: obfuscation,
            selectNearbyLocation: tunnelSettings.automaticMultihopIsEnabled
        )
    }

    func exclude(
        relay: SelectedRelay,
        from candidates: [RelayWithLocation<REST.ServerRelay>],
        closeTo location: Location? = nil,
        obfuscation: RelayObfuscation?,
        forceV4Address: Bool = false,
    ) throws -> SelectedRelay {
        let filteredCandidates = candidates.filter { relayWithLocation in
            relayWithLocation.relay.hostname != relay.hostname
        }

        return try findBestMatch(
            from: filteredCandidates,
            closeTo: location,
            obfuscation: obfuscation,
            forceV4Address: forceV4Address,
        )
    }
}
