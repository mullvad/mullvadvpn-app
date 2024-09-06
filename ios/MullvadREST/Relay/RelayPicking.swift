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
    var daitaSettings: DAITASettings { get }
    func pick() throws -> SelectedRelays
}

extension RelayPicking {
    func findBestMatch(
        from candidates: [RelayWithLocation<REST.ServerRelay>],
        closeTo location: Location? = nil
    ) throws -> SelectedRelay {
        let match = try RelaySelector.WireGuard.pickCandidate(
            from: candidates,
            relays: relays,
            portConstraint: constraints.port,
            numberOfFailedAttempts: connectionAttemptCount,
            closeTo: location
        )

        return SelectedRelay(
            endpoint: match.endpoint,
            hostname: match.relay.hostname,
            location: match.location
        )
    }
}

struct SinglehopPicker: RelayPicking {
    let relays: REST.ServerRelaysResponse
    let constraints: RelayConstraints
    let connectionAttemptCount: UInt
    let daitaSettings: DAITASettings

    func pick() throws -> SelectedRelays {
        do {
            let exitCandidates = try RelaySelector.WireGuard.findCandidates(
                by: constraints.exitLocations,
                in: relays,
                filterConstraint: constraints.filter,
                daitaEnabled: daitaSettings.daitaState.isEnabled
            )

            let match = try findBestMatch(from: exitCandidates)
            return SelectedRelays(entry: nil, exit: match, retryAttempt: connectionAttemptCount)
        } catch let error as NoRelaysSatisfyingConstraintsError where error.reason == .noDaitaRelaysFound {
            // If DAITA and smart routing are enabled and no supported relays are found, we should try to find the nearest
            // available relay that supports DAITA and use it as entry in a multihop selection.
            if daitaSettings.shouldDoAutomaticRouting {
                var constraints = constraints
                constraints.entryLocations = .any

                return try MultihopPicker(
                    relays: relays,
                    constraints: constraints,
                    connectionAttemptCount: connectionAttemptCount,
                    daitaSettings: daitaSettings,
                    automaticDaitaRouting: true
                ).pick()
            } else {
                throw error
            }
        }
    }
}

struct MultihopPicker: RelayPicking {
    let relays: REST.ServerRelaysResponse
    let constraints: RelayConstraints
    let connectionAttemptCount: UInt
    let daitaSettings: DAITASettings
    let automaticDaitaRouting: Bool

    func pick() throws -> SelectedRelays {
        let exitCandidates = try RelaySelector.WireGuard.findCandidates(
            by: constraints.exitLocations,
            in: relays,
            filterConstraint: constraints.filter,
            daitaEnabled: false
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

        do {
            let entryCandidates = try RelaySelector.WireGuard.findCandidates(
                by: constraints.entryLocations,
                in: relays,
                filterConstraint: constraints.filter,
                daitaEnabled: daitaSettings.daitaState.isEnabled
            )

            return try decisionFlow.pick(
                entryCandidates: entryCandidates,
                exitCandidates: exitCandidates,
                automaticDaitaRouting: automaticDaitaRouting
            )
        } catch let error as NoRelaysSatisfyingConstraintsError where error.reason == .noDaitaRelaysFound {
            // If DAITA and smart routing are enabled and no supported relays are found, we should try to find the nearest
            // available relay that supports DAITA and use it as entry in a multihop selection.
            if daitaSettings.shouldDoAutomaticRouting {
                let entryCandidates = try RelaySelector.WireGuard.findCandidates(
                    by: .any,
                    in: relays,
                    filterConstraint: constraints.filter,
                    daitaEnabled: true
                )

                return try decisionFlow.pick(
                    entryCandidates: entryCandidates,
                    exitCandidates: exitCandidates,
                    automaticDaitaRouting: true
                )
            } else {
                throw error
            }
        }
    }

    func exclude(
        relay: SelectedRelay,
        from candidates: [RelayWithLocation<REST.ServerRelay>],
        closeTo location: Location? = nil
    ) throws -> SelectedRelay {
        let filteredCandidates = candidates.filter { relayWithLocation in
            relayWithLocation.relay.hostname != relay.hostname
        }

        return try findBestMatch(from: filteredCandidates, closeTo: location)
    }
}
