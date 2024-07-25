//
//  RelaySelectorPicker.swift
//  MullvadREST
//
//  Created by Jon Petersson on 2024-06-05.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import MullvadTypes

protocol RelayPicking {
    var relays: REST.ServerRelaysResponse { get }
    var constraints: RelayConstraints { get }
    var connectionAttemptCount: UInt { get }
    func pick() throws -> SelectedRelays
}

extension RelayPicking {
    func findBestMatch(
        from candidates: [RelayWithLocation<REST.ServerRelay>]
    ) throws -> SelectedRelay {
        let match = try RelaySelector.WireGuard.pickCandidate(
            from: candidates,
            relays: relays,
            portConstraint: constraints.port,
            numberOfFailedAttempts: connectionAttemptCount
        )

        return SelectedRelay(
            endpoint: match.endpoint,
            hostname: match.relay.hostname,
            location: match.location
        )
    }
}

struct SinglehopPicker: RelayPicking {
    let constraints: RelayConstraints
    let relays: REST.ServerRelaysResponse
    let connectionAttemptCount: UInt

    func pick() throws -> SelectedRelays {
        let candidates = try RelaySelector.WireGuard.findCandidates(
            by: constraints.exitLocations,
            in: relays,
            filterConstraint: constraints.filter
        )

        let match = try findBestMatch(from: candidates)

        return SelectedRelays(entry: nil, exit: match, retryAttempt: connectionAttemptCount)
    }
}

struct MultihopPicker: RelayPicking {
    let constraints: RelayConstraints
    let relays: REST.ServerRelaysResponse
    let connectionAttemptCount: UInt

    func pick() throws -> SelectedRelays {
        let entryCandidates = try RelaySelector.WireGuard.findCandidates(
            by: constraints.entryLocations,
            in: relays,
            filterConstraint: constraints.filter
        )

        let exitCandidates = try RelaySelector.WireGuard.findCandidates(
            by: constraints.exitLocations,
            in: relays,
            filterConstraint: constraints.filter
        )

        /*
         Relay selection is prioritised in the following order:
         1. Both entry and exit constraints match only a single relay. Both relays are selected.
         2. Either entry or exit constraint matches only a single relay and the other multiple relays. The single relays
         is selected and excluded from the list of multiple relays.
         3. Both entry and exit constraints match multiple relays. Exit relay is picked first and then excluded from
         the list of entry relays.
         */
        let decisionFlow = OneToOne(
            next: OneToMany(
                next: ManyToMany(
                    next: nil,
                    relayPicker: self
                ),
                relayPicker: self
            ),
            relayPicker: self
        )

        return try decisionFlow.pick(entryCandidates: entryCandidates, exitCandidates: exitCandidates)
    }

    func exclude(
        relay: SelectedRelay,
        from candidates: [RelayWithLocation<REST.ServerRelay>]
    ) throws -> SelectedRelay {
        let filteredCandidates = candidates.filter { relayWithLocation in
            relayWithLocation.relay.hostname != relay.hostname
        }

        return try findBestMatch(from: filteredCandidates)
    }
}
