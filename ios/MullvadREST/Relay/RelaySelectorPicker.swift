//
//  RelaySelectorPicker.swift
//  MullvadREST
//
//  Created by Jon Petersson on 2024-06-05.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import MullvadTypes

protocol RelaySelectorPicker {
    var relays: REST.ServerRelaysResponse { get }
    var constraints: RelayConstraints { get }
    var connectionAttemptCount: UInt { get }
    func pick() throws -> SelectedRelays
}

extension RelaySelectorPicker {
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
            location: match.location,
            retryAttempts: connectionAttemptCount
        )
    }
}

struct SinglehopPicker: RelaySelectorPicker {
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

        return SelectedRelays(entry: nil, exit: match)
    }
}

struct MultihopPicker: RelaySelectorPicker {
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

        let decisionChain = OneToOne(
            next: OneToMany(next: ManyToMany(next: nil, relaySelectorPicker: self), relaySelectorPicker: self),
            relaySelectorPicker: self
        )

        return try decisionChain.pick(entryCandidates: entryCandidates, exitCandidates: exitCandidates)
    }

    func exclude(
        relay: SelectedRelay,
        from candidates: [RelayWithLocation<REST.ServerRelay>]
    ) throws -> SelectedRelay {
        let filteredCandidates = candidates.filter { relayWithLocation in
            relayWithLocation.serverLocation != relay.location
        }

        return try findBestMatch(from: filteredCandidates)
    }
}

protocol MultihopDescionMaker {
    typealias RelayCandidate = RelayWithLocation<REST.ServerRelay>
    init(next: MultihopDescionMaker?, relaySelectorPicker: RelaySelectorPicker)
    func canHandle(entryCandidates: [RelayCandidate], exitCandidates: [RelayCandidate]) -> Bool
    func pick(entryCandidates: [RelayCandidate], exitCandidates: [RelayCandidate]) throws -> SelectedRelays
}

private struct OneToOne: MultihopDescionMaker {
    let next: MultihopDescionMaker?
    let relaySelectorPicker: RelaySelectorPicker
    init(next: (any MultihopDescionMaker)?, relaySelectorPicker: RelaySelectorPicker) {
        self.next = next
        self.relaySelectorPicker = relaySelectorPicker
    }

    func pick(entryCandidates: [RelayCandidate], exitCandidates: [RelayCandidate]) throws -> SelectedRelays {
        guard canHandle(entryCandidates: entryCandidates, exitCandidates: exitCandidates) else {
            guard let next else {
                throw NoRelaysSatisfyingConstraintsError()
            }
            return try next.pick(entryCandidates: entryCandidates, exitCandidates: exitCandidates)
        }

        guard entryCandidates.first != exitCandidates.first else {
            throw NoRelaysSatisfyingConstraintsError()
        }

        let entryMatch = try relaySelectorPicker.findBestMatch(from: entryCandidates)
        let exitMatch = try relaySelectorPicker.findBestMatch(from: exitCandidates)
        return SelectedRelays(entry: entryMatch, exit: exitMatch)
    }

    func canHandle(entryCandidates: [RelayCandidate], exitCandidates: [RelayCandidate]) -> Bool {
        entryCandidates.count == 1 && exitCandidates.count == 1
    }
}

private struct OneToMany: MultihopDescionMaker {
    let next: MultihopDescionMaker?
    let relaySelectorPicker: RelaySelectorPicker

    init(next: (any MultihopDescionMaker)?, relaySelectorPicker: RelaySelectorPicker) {
        self.next = next
        self.relaySelectorPicker = relaySelectorPicker
    }

    func pick(entryCandidates: [RelayCandidate], exitCandidates: [RelayCandidate]) throws -> SelectedRelays {
        guard let multihopPicker = relaySelectorPicker as? MultihopPicker else {
            fatalError("Could not cast picker to MultihopPicker")
        }

        guard canHandle(entryCandidates: entryCandidates, exitCandidates: exitCandidates) else {
            guard let next else {
                throw NoRelaysSatisfyingConstraintsError()
            }
            return try next.pick(entryCandidates: entryCandidates, exitCandidates: exitCandidates)
        }

        switch (entryCandidates.count, exitCandidates.count) {
        case let (1, count) where count > 1:
            let entryMatch = try multihopPicker.findBestMatch(from: entryCandidates)
            let exitMatch = try multihopPicker.exclude(relay: entryMatch, from: exitCandidates)
            return SelectedRelays(entry: entryMatch, exit: exitMatch)
        default:
            let exitMatch = try multihopPicker.findBestMatch(from: exitCandidates)
            let entryMatch = try multihopPicker.exclude(relay: exitMatch, from: entryCandidates)
            return SelectedRelays(entry: entryMatch, exit: exitMatch)
        }
    }

    func canHandle(entryCandidates: [RelayCandidate], exitCandidates: [RelayCandidate]) -> Bool {
        (entryCandidates.count == 1 && exitCandidates.count > 1) ||
            (entryCandidates.count > 1 && exitCandidates.count == 1)
    }
}

private struct ManyToMany: MultihopDescionMaker {
    let next: MultihopDescionMaker?
    let relaySelectorPicker: RelaySelectorPicker

    init(next: (any MultihopDescionMaker)?, relaySelectorPicker: RelaySelectorPicker) {
        self.next = next
        self.relaySelectorPicker = relaySelectorPicker
    }

    func pick(entryCandidates: [RelayCandidate], exitCandidates: [RelayCandidate]) throws -> SelectedRelays {
        guard let multihopPicker = relaySelectorPicker as? MultihopPicker else {
            fatalError("Could not cast picker to MultihopPicker")
        }

        guard canHandle(entryCandidates: entryCandidates, exitCandidates: exitCandidates) else {
            guard let next else {
                throw NoRelaysSatisfyingConstraintsError()
            }
            return try next.pick(entryCandidates: entryCandidates, exitCandidates: exitCandidates)
        }

        let exitMatch = try multihopPicker.findBestMatch(from: exitCandidates)
        let entryMatch = try multihopPicker.exclude(relay: exitMatch, from: entryCandidates)
        return SelectedRelays(entry: entryMatch, exit: exitMatch)
    }

    func canHandle(entryCandidates: [RelayCandidate], exitCandidates: [RelayCandidate]) -> Bool {
        entryCandidates.count > 1 && exitCandidates.count > 1
    }
}
