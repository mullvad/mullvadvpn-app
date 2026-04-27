//
//  MultihopDecisionFlow.swift
//  MullvadREST
//
//  Created by Jon Petersson on 2024-06-14.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation

protocol MultihopDecisionFlow {
    typealias RelayCandidate = RelayWithLocation<REST.ServerRelay>
    init(next: MultihopDecisionFlow?, relayPicker: RelayPicking)
    func canHandle(entryCandidates: [RelayCandidate], exitCandidates: [RelayCandidate]) -> Bool
    func pick(
        entryCandidates: [RelayCandidate],
        exitCandidates: [RelayCandidate],
        obfuscation: RelayObfuscation,
        selectNearbyLocation: Bool
    ) throws -> SelectedRelays
}

struct OneToOne: MultihopDecisionFlow {
    let next: MultihopDecisionFlow?
    let relayPicker: RelayPicking

    init(next: (any MultihopDecisionFlow)?, relayPicker: RelayPicking) {
        self.next = next
        self.relayPicker = relayPicker
    }

    func pick(
        entryCandidates: [RelayCandidate],
        exitCandidates: [RelayCandidate],
        obfuscation: RelayObfuscation,
        selectNearbyLocation: Bool
    ) throws -> SelectedRelays {
        guard canHandle(entryCandidates: entryCandidates, exitCandidates: exitCandidates) else {
            guard let next else {
                throw NoRelaysSatisfyingConstraintsError(.multihopInvalidFlow)
            }
            return try next.pick(
                entryCandidates: entryCandidates,
                exitCandidates: exitCandidates,
                obfuscation: obfuscation,
                selectNearbyLocation: selectNearbyLocation
            )
        }

        guard entryCandidates.first != exitCandidates.first else {
            throw NoRelaysSatisfyingConstraintsError(.entryEqualsExit)
        }

        let exitMatch = try relayPicker.findBestMatch(from: exitCandidates, obfuscation: nil, forceV4Address: true)
        let entryMatch = try relayPicker.findBestMatch(
            from: entryCandidates,
            closeTo: selectNearbyLocation ? exitMatch.location : nil,
            obfuscation: obfuscation
        )

        return SelectedRelays(
            entry: entryMatch,
            exit: exitMatch,
            retryAttempt: relayPicker.connectionAttemptCount
        )
    }

    func canHandle(entryCandidates: [RelayCandidate], exitCandidates: [RelayCandidate]) -> Bool {
        entryCandidates.count == 1 && exitCandidates.count == 1
    }
}

struct OneToMany: MultihopDecisionFlow {
    let next: MultihopDecisionFlow?
    let relayPicker: RelayPicking

    init(next: (any MultihopDecisionFlow)?, relayPicker: RelayPicking) {
        self.next = next
        self.relayPicker = relayPicker
    }

    func pick(
        entryCandidates: [RelayCandidate],
        exitCandidates: [RelayCandidate],
        obfuscation: RelayObfuscation,
        selectNearbyLocation: Bool
    ) throws -> SelectedRelays {
        guard let multihopPicker = relayPicker as? MultihopPicker else {
            fatalError("Could not cast picker to MultihopPicker")
        }

        guard canHandle(entryCandidates: entryCandidates, exitCandidates: exitCandidates) else {
            guard let next else {
                throw NoRelaysSatisfyingConstraintsError(.multihopInvalidFlow)
            }
            return try next.pick(
                entryCandidates: entryCandidates,
                exitCandidates: exitCandidates,
                obfuscation: obfuscation,
                selectNearbyLocation: selectNearbyLocation
            )
        }

        let entryMatch = try multihopPicker.findBestMatch(from: entryCandidates, obfuscation: obfuscation)
        let exitMatch = try multihopPicker.exclude(
            relay: entryMatch,
            from: exitCandidates,
            obfuscation: nil,
            forceV4Address: true,
        )

        return SelectedRelays(
            entry: entryMatch,
            exit: exitMatch,
            retryAttempt: relayPicker.connectionAttemptCount
        )
    }

    func canHandle(entryCandidates: [RelayCandidate], exitCandidates: [RelayCandidate]) -> Bool {
        entryCandidates.count == 1 && exitCandidates.count > 1
    }
}

struct ManyToOne: MultihopDecisionFlow {
    let next: MultihopDecisionFlow?
    let relayPicker: RelayPicking

    init(next: (any MultihopDecisionFlow)?, relayPicker: RelayPicking) {
        self.next = next
        self.relayPicker = relayPicker
    }

    func pick(
        entryCandidates: [RelayCandidate],
        exitCandidates: [RelayCandidate],
        obfuscation: RelayObfuscation,
        selectNearbyLocation: Bool
    ) throws -> SelectedRelays {
        guard let multihopPicker = relayPicker as? MultihopPicker else {
            fatalError("Could not cast picker to MultihopPicker")
        }

        guard canHandle(entryCandidates: entryCandidates, exitCandidates: exitCandidates) else {
            guard let next else {
                throw NoRelaysSatisfyingConstraintsError(.multihopInvalidFlow)
            }
            return try next.pick(
                entryCandidates: entryCandidates,
                exitCandidates: exitCandidates,
                obfuscation: obfuscation,
                selectNearbyLocation: selectNearbyLocation
            )
        }

        let exitMatch = try multihopPicker.findBestMatch(
            from: exitCandidates,
            obfuscation: nil,
            forceV4Address: true,
        )
        let entryMatch = try multihopPicker.exclude(
            relay: exitMatch,
            from: entryCandidates,
            closeTo: selectNearbyLocation ? exitMatch.location : nil,
            obfuscation: obfuscation
        )

        return SelectedRelays(
            entry: entryMatch,
            exit: exitMatch,
            retryAttempt: relayPicker.connectionAttemptCount
        )
    }

    func canHandle(entryCandidates: [RelayCandidate], exitCandidates: [RelayCandidate]) -> Bool {
        entryCandidates.count > 1 && exitCandidates.count == 1
    }
}

struct ManyToMany: MultihopDecisionFlow {
    let next: MultihopDecisionFlow?
    let relayPicker: RelayPicking

    init(next: (any MultihopDecisionFlow)?, relayPicker: RelayPicking) {
        self.next = next
        self.relayPicker = relayPicker
    }

    func pick(
        entryCandidates: [RelayCandidate],
        exitCandidates: [RelayCandidate],
        obfuscation: RelayObfuscation,
        selectNearbyLocation: Bool
    ) throws -> SelectedRelays {
        guard let multihopPicker = relayPicker as? MultihopPicker else {
            fatalError("Could not cast picker to MultihopPicker")
        }

        guard canHandle(entryCandidates: entryCandidates, exitCandidates: exitCandidates) else {
            guard let next else {
                throw NoRelaysSatisfyingConstraintsError(.multihopInvalidFlow)
            }
            return try next.pick(
                entryCandidates: entryCandidates,
                exitCandidates: exitCandidates,
                obfuscation: obfuscation,
                selectNearbyLocation: selectNearbyLocation
            )
        }

        let exitMatch = try multihopPicker.findBestMatch(from: exitCandidates, obfuscation: nil, forceV4Address: true)
        let entryMatch = try multihopPicker.exclude(
            relay: exitMatch,
            from: entryCandidates,
            closeTo: selectNearbyLocation ? exitMatch.location : nil,
            obfuscation: obfuscation
        )

        return SelectedRelays(
            entry: entryMatch,
            exit: exitMatch,
            retryAttempt: relayPicker.connectionAttemptCount
        )
    }

    func canHandle(entryCandidates: [RelayCandidate], exitCandidates: [RelayCandidate]) -> Bool {
        entryCandidates.count > 1 && exitCandidates.count > 1
    }
}
