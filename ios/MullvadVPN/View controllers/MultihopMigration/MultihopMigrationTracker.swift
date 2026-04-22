//
//  MultihopMigrationTracker.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-04-01.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadREST
import MullvadSettings
import MullvadTypes

struct MultihopRule<Output>: Sendable {
    let name: String
    let condition: @Sendable (LatestTunnelSettings) -> Bool
    let transform: @Sendable (inout LatestTunnelSettings) -> MigrationOutput<Output>
}

enum SettingsUpdate {
    case none
    case updatedMultiHop
    case uniqueFilter
    case directOnlyRemoved
    case automatic
}

struct Change {
    let path: SettingsUpdate
    var before: Any? = nil
    var after: Any? = nil
}

struct MultihopActionSuggestion: Sendable {
    let name: String
    let action: (@Sendable (inout LatestTunnelSettings) -> ()?)?
}

struct MigrationOutput<T> {
    let value: T
    let changes: [Change]
    var action: MultihopActionSuggestion? = nil
}

enum MigrationError: Error {
    case noMatchingRule
}

struct MultihopMigrationTracker<Output> {
    let rules: [MultihopRule<Output>]

    func run(input: inout LatestTunnelSettings) throws -> MigrationOutput<Output> {
        for rule in rules {
            if rule.condition(input) {
                return rule.transform(&input)
            }
        }

        throw MigrationError.noMatchingRule
    }
}

enum MultihopMigrationTrackerFactory {
    static func make(_ relaySelector: RelaySelectorProtocol) -> MultihopMigrationTracker<MultihopStateV2> {

        let scenario1A = MultihopRule<MultihopStateV2>(
            name: "Scenario1 A"
        ) { input in
            input.tunnelMultihopState == .never && input.daita.daitaState == .off
                && input.relayConstraints.exitFilter == .any
        } transform: { input in

            let newValue: MultihopStateV2 = .whenNeeded
            input.tunnelMultihopState = newValue

            return MigrationOutput(
                value: newValue,
                changes: [
                    Change(path: .updatedMultiHop, before: input.tunnelMultihopState, after: newValue)
                ]
            )
        }

        let scenario1B = MultihopRule<MultihopStateV2>(
            name: "Scenario1 B"
        ) { input in
            input.tunnelMultihopState == .never && input.daita.daitaState == .off
                && input.relayConstraints.exitFilter != .any
        } transform: { input in

            let newValue: MultihopStateV2 = .never

            return MigrationOutput(
                value: newValue,
                changes: [
                    Change(path: .updatedMultiHop, before: MultihopStateV1.off, after: newValue),
                    Change(path: .uniqueFilter, before: nil, after: input.relayConstraints.exitFilter),
                ],
                action: MultihopActionSuggestion(
                    name: String(
                        format: NSLocalizedString("Change to “%@”", comment: ""),
                        arguments: [NSLocalizedString(MultihopStateV2.whenNeeded.description, comment: "")]),
                    action: { latestTunnelSettings in
                        latestTunnelSettings.tunnelMultihopState = .whenNeeded
                    })
            )
        }

        let scenario2 = MultihopRule<MultihopStateV2>(
            name: "Scenario2"
        ) { input in
            input.tunnelMultihopState == .never && input.daita.daitaState == .on && input.daita.directOnlyState == .off
                && input.relayConstraints.exitFilter == .any
        } transform: { input in

            let newValue: MultihopStateV2 = .whenNeeded
            input.tunnelMultihopState = newValue

            return MigrationOutput(
                value: newValue,
                changes: [
                    Change(path: .updatedMultiHop, before: MultihopStateV1.off, after: newValue)
                ]
            )
        }

        let scenario3A = MultihopRule<MultihopStateV2>(
            name: "Scenario 3A"
        ) { input in
            var isMultihopNeeded = false
            do {
                _ = try relaySelector.selectRelays(
                    tunnelSettings: input,
                    connectionAttemptCount: 0
                )
            } catch {
                isMultihopNeeded = true
            }
            return input.tunnelMultihopState == .never && input.daita.daitaState == .on
                && input.daita.directOnlyState == .off
                && input.relayConstraints.exitFilter != .any && !isMultihopNeeded
        } transform: { input in
            input.tunnelMultihopState = .never
            return MigrationOutput(
                value: .never,
                changes: [
                    Change(path: .uniqueFilter, before: nil, after: input.relayConstraints.entryFilter),
                    Change(path: .updatedMultiHop, before: MultihopStateV1.off, after: MultihopStateV2.never),
                ],
                action: MultihopActionSuggestion(
                    name: String(
                        format: NSLocalizedString("Change to ”%@”", comment: ""),
                        arguments: [MultihopStateV2.whenNeeded.description]),
                    action: { latestTunnelSettings in
                        latestTunnelSettings.tunnelMultihopState = .whenNeeded
                    }))
        }

        let scenario3B = MultihopRule<MultihopStateV2>(
            name: "Scenario 3B"
        ) { input in

            var isMultihopNeeded = false
            do {
                _ = try relaySelector.selectRelays(
                    tunnelSettings: input,
                    connectionAttemptCount: 0
                )
            } catch {
                isMultihopNeeded = true
            }
            return input.tunnelMultihopState == .never && input.daita.daitaState == .on
                && input.daita.directOnlyState == .off
                && input.relayConstraints.exitFilter != .any && isMultihopNeeded
        } transform: { input in
            input.tunnelMultihopState = .always
            return MigrationOutput(
                value: .always,
                changes: [
                    Change(path: .uniqueFilter, before: nil, after: input.relayConstraints.entryFilter),
                    Change(path: .updatedMultiHop, before: MultihopStateV1.off, after: MultihopStateV2.always),
                ],
                action: MultihopActionSuggestion(
                    name: String(
                        format: NSLocalizedString("Set entry to “%@”", comment: ""),
                        arguments: [NSLocalizedString("Automatic", comment: "")]),
                    action: { latestTunnelSettings in
                        latestTunnelSettings.relayConstraints.entryLocations = .any
                    }))
        }

        let scenario4A = MultihopRule<MultihopStateV2>(
            name: "Scenario 4A"
        ) { input in
            input.tunnelMultihopState == .never && input.daita.daitaState == .on && input.daita.directOnlyState == .on
                && input.relayConstraints.exitFilter == .any
        } transform: { input in

            let newValue: MultihopStateV2 = .never
            input.tunnelMultihopState = newValue

            return MigrationOutput(
                value: newValue,
                changes: [
                    Change(path: .directOnlyRemoved),
                    Change(path: .updatedMultiHop, before: MultihopStateV1.off, after: newValue),
                ]
            )
        }

        let scenario4B = MultihopRule<MultihopStateV2>(
            name: "Scenario 4B"
        ) { input in
            input.tunnelMultihopState == .never && input.daita.daitaState == .on && input.daita.directOnlyState == .on
                && input.relayConstraints.exitFilter != .any
        } transform: { input in

            let newValue: MultihopStateV2 = .never
            input.tunnelMultihopState = newValue

            return MigrationOutput(
                value: newValue,
                changes: [
                    Change(path: .directOnlyRemoved, before: nil, after: nil),
                    Change(path: .updatedMultiHop, before: MultihopStateV1.off, after: newValue),
                ]
            )
        }

        let scenario5A = MultihopRule<MultihopStateV2>(
            name: "Scenario 5A"
        ) { input in
            input.tunnelMultihopState == .always && input.daita.daitaState == .off
                && input.relayConstraints.entryFilter == .any
        } transform: { input in

            let newValue: MultihopStateV2 = .always

            return MigrationOutput(
                value: newValue,
                changes: [
                    Change(path: .updatedMultiHop, before: MultihopStateV1.on, after: newValue)
                ]
            )
        }

        let scenario5B = MultihopRule<MultihopStateV2>(
            name: "Scenario 5B"
        ) { input in
            input.tunnelMultihopState == .always && input.daita.daitaState == .off
                && input.relayConstraints.entryFilter != .any
        } transform: { input in

            let newValue: MultihopStateV2 = .always

            return MigrationOutput(
                value: newValue,
                changes: [
                    Change(path: .updatedMultiHop, before: MultihopStateV1.on, after: newValue),
                    Change(path: .uniqueFilter),
                ]
            )
        }

        let scenario6A = MultihopRule<MultihopStateV2>(
            name: "Scenario 6A"
        ) { input in

            input.tunnelMultihopState == .always && input.daita.daitaState == .on && input.daita.directOnlyState == .off
                && input.relayConstraints.entryFilter == .any
        } transform: { input in

            let newValue: MultihopStateV2 = .always
            input.tunnelMultihopState = newValue

            return MigrationOutput(
                value: newValue,
                changes: [
                    Change(path: .updatedMultiHop, before: MultihopStateV1.on, after: newValue)
                ]
            )
        }

        let scenario6B = MultihopRule<MultihopStateV2>(
            name: "Scenario 6B"
        ) { input in

            input.tunnelMultihopState == .always && input.daita.daitaState == .on && input.daita.directOnlyState == .off
                && input.relayConstraints.entryFilter != .any
        } transform: { input in

            let newValue: MultihopStateV2 = .always
            input.tunnelMultihopState = newValue

            return MigrationOutput(
                value: newValue,
                changes: [
                    Change(path: .updatedMultiHop, before: MultihopStateV1.on, after: newValue),
                    Change(path: .uniqueFilter),
                ],
                action: MultihopActionSuggestion(
                    name: String(
                        format: NSLocalizedString("Set entry to “%@”", comment: ""),
                        arguments: [NSLocalizedString("Automatic", comment: "")]),
                    action: { latestTunnelSettings in
                        latestTunnelSettings.relayConstraints.entryLocations = .any
                    }))
        }
        let scenario7A = MultihopRule<MultihopStateV2>(
            name: "Scenario 7A"
        ) { input in

            input.tunnelMultihopState == .always && input.daita.daitaState == .on && input.daita.directOnlyState == .on
                && input.relayConstraints.entryFilter == .any
        } transform: { input in

            let newValue: MultihopStateV2 = .always
            input.tunnelMultihopState = newValue

            return MigrationOutput(
                value: newValue,
                changes: [
                    Change(path: .directOnlyRemoved, before: nil, after: nil),
                    Change(path: .updatedMultiHop, before: MultihopStateV1.on, after: newValue),
                ])
        }

        let scenario7B = MultihopRule<MultihopStateV2>(
            name: "Scenario 7B"
        ) { input in

            input.tunnelMultihopState == .always && input.daita.daitaState == .on && input.daita.directOnlyState == .on
                && input.relayConstraints.entryFilter != .any
        } transform: { input in

            let newValue: MultihopStateV2 = .always
            input.tunnelMultihopState = newValue

            return MigrationOutput(
                value: newValue,
                changes: [
                    Change(path: .directOnlyRemoved, before: nil, after: nil),
                    Change(path: .updatedMultiHop, before: MultihopStateV1.on, after: newValue),
                    Change(path: .uniqueFilter),
                ],
                action: MultihopActionSuggestion(
                    name: String(
                        format: NSLocalizedString("Set entry to “%@”", comment: ""),
                        arguments: [NSLocalizedString("Automatic", comment: "")]),
                    action: { latestTunnelSettings in
                        latestTunnelSettings.relayConstraints.entryLocations = .any
                    }))
        }

        return MultihopMigrationTracker(rules: [
            scenario1A, scenario1B,
            scenario2,
            scenario3A, scenario3B,
            scenario4A, scenario4B,
            scenario5A, scenario5B,
            scenario6A, scenario6B,
            scenario7A, scenario7B,
        ])
    }
}
