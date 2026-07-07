//
//  MultihopMigrationTrackerFactory.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-05-08.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadREST
import MullvadSettings
import MullvadTypes

enum MultihopSuggestedAction: Sendable {
    case multihopWhenNeeded
    case automaticEntry
}

enum MultihopMigrationTrackerFactory {
    static func make(_ relaySelector: RelaySelectorProtocol) -> SettingsMigrationTracker<
        MultihopStateV2, MultihopSuggestedAction
    > {

        let scenario1A = SettingsRule<MultihopStateV2, MultihopSuggestedAction>(
            name: "Scenario 1A"
        ) { input in
            input.tunnelMultihopState == .never && !input.daita.isEnabled
                && input.relayConstraints.exitFilter == .any
        } transform: { input in

            let newValue: MultihopStateV2 = .whenNeeded
            input.tunnelMultihopState = newValue

            return MigrationResult(
                changes: []
            )
        }

        let scenario1B = SettingsRule<MultihopStateV2, MultihopSuggestedAction>(
            name: "Scenario 1B"
        ) { input in
            input.tunnelMultihopState == .never && !input.daita.isEnabled
                && input.relayConstraints.exitFilter != .any
        } transform: { input in

            let newValue: MultihopStateV2 = .never

            return MigrationResult(
                changes: [
                    Change(path: .updatedMultiHop, before: MultihopStateV1.off, after: newValue),
                    Change(path: .uniqueFilter, before: nil, after: input.relayConstraints.exitFilter),
                ],
                action: SuggestedAction(
                    kind: MultihopSuggestedAction.multihopWhenNeeded,
                    action: { latestTunnelSettings in
                        latestTunnelSettings.tunnelMultihopState = .whenNeeded
                    })
            )
        }

        let scenario2 = SettingsRule<MultihopStateV2, MultihopSuggestedAction>(
            name: "Scenario 2"
        ) { input in
            input.tunnelMultihopState == .never && input.daita.isEnabled && !input.daita.directOnlyState.isEnabled
                && input.relayConstraints.exitFilter == .any
        } transform: { input in

            let newValue: MultihopStateV2 = .whenNeeded
            input.tunnelMultihopState = newValue

            return MigrationResult(
                changes: [
                    Change(path: .updatedMultiHop, before: MultihopStateV1.off, after: newValue)
                ]
            )
        }

        let scenario3A = SettingsRule<MultihopStateV2, MultihopSuggestedAction>(
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
            return input.tunnelMultihopState == .never && input.daita.isEnabled
                && !input.daita.directOnlyState.isEnabled
                && input.relayConstraints.exitFilter != .any && !isMultihopNeeded
        } transform: { input in
            input.tunnelMultihopState = .never
            return MigrationResult(
                changes: [
                    Change(path: .updatedMultiHop, before: MultihopStateV1.off, after: MultihopStateV2.never),
                    Change(path: .uniqueFilter, before: nil, after: input.relayConstraints.entryFilter),
                ],
                action: SuggestedAction(
                    kind: MultihopSuggestedAction.multihopWhenNeeded,
                    action: { latestTunnelSettings in
                        latestTunnelSettings.tunnelMultihopState = .whenNeeded
                    }))
        }

        let scenario3B = SettingsRule<MultihopStateV2, MultihopSuggestedAction>(
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
            return input.tunnelMultihopState == .never && input.daita.isEnabled
                && !input.daita.directOnlyState.isEnabled
                && input.relayConstraints.exitFilter != .any && isMultihopNeeded
        } transform: { input in

            // Find compatible relays within the country before enabling changing mode to avoid blocking the connection.
            input.tunnelMultihopState = .whenNeeded
            let suggestedLocations = try? relaySelector.selectRelays(tunnelSettings: input, connectionAttemptCount: 0)
            if let suggestedEntry = suggestedLocations?.entry {
                let entryConstraints = UserSelectedRelays(locations: [.country(suggestedEntry.location.countryCode)])
                input.relayConstraints.entryLocations = .only(entryConstraints)
            }

            input.tunnelMultihopState = .always

            return MigrationResult(
                changes: [
                    Change(path: .updatedMultiHop, before: MultihopStateV1.off, after: MultihopStateV2.always),
                    Change(path: .uniqueFilter, before: nil, after: input.relayConstraints.entryFilter),
                ],
                action: SuggestedAction(
                    kind: MultihopSuggestedAction.automaticEntry,
                    action: { latestTunnelSettings in
                        latestTunnelSettings.relayConstraints.entryLocations = .any
                    }))
        }

        let scenario4A = SettingsRule<MultihopStateV2, MultihopSuggestedAction>(
            name: "Scenario 4A"
        ) { input in
            input.tunnelMultihopState == .never && input.daita.isEnabled && input.daita.directOnlyState.isEnabled
                && input.relayConstraints.exitFilter == .any
        } transform: { input in

            let newValue: MultihopStateV2 = .never
            input.tunnelMultihopState = newValue

            return MigrationResult(
                changes: [
                    Change(path: .updatedMultiHop, before: MultihopStateV1.off, after: newValue),
                    Change(path: .directOnlyRemoved),
                ]
            )
        }

        let scenario4B = SettingsRule<MultihopStateV2, MultihopSuggestedAction>(
            name: "Scenario 4B"
        ) { input in
            input.tunnelMultihopState == .never && input.daita.isEnabled && input.daita.directOnlyState.isEnabled
                && input.relayConstraints.exitFilter != .any
        } transform: { input in

            let newValue: MultihopStateV2 = .never
            input.tunnelMultihopState = newValue

            return MigrationResult(
                changes: [
                    Change(path: .updatedMultiHop, before: MultihopStateV1.off, after: newValue),
                    Change(path: .directOnlyRemoved),
                    Change(path: .uniqueFilter),
                ]
            )
        }

        let scenario5A = SettingsRule<MultihopStateV2, MultihopSuggestedAction>(
            name: "Scenario 5A"
        ) { input in
            input.tunnelMultihopState == .always && !input.daita.isEnabled
                && input.relayConstraints.entryFilter == .any
        } transform: { input in

            let newValue: MultihopStateV2 = .always

            return MigrationResult(
                changes: [
                    Change(path: .updatedMultiHop, before: MultihopStateV1.on, after: newValue)
                ]
            )
        }

        let scenario5B = SettingsRule<MultihopStateV2, MultihopSuggestedAction>(
            name: "Scenario 5B"
        ) { input in
            input.tunnelMultihopState == .always && !input.daita.isEnabled
                && input.relayConstraints.entryFilter != .any
        } transform: { input in

            let newValue: MultihopStateV2 = .always

            return MigrationResult(
                changes: [
                    Change(path: .updatedMultiHop, before: MultihopStateV1.on, after: newValue),
                    Change(path: .uniqueFilter),
                ]
            )
        }

        let scenario6A = SettingsRule<MultihopStateV2, MultihopSuggestedAction>(
            name: "Scenario 6A"
        ) { input in
            input.tunnelMultihopState == .always && input.daita.isEnabled && !input.daita.directOnlyState.isEnabled
                && input.relayConstraints.entryFilter == .any
        } transform: { input in

            let newValue: MultihopStateV2 = .always
            input.tunnelMultihopState = newValue
            input.relayConstraints.entryLocations = .any

            return MigrationResult(
                changes: [
                    Change(path: .updatedMultiHop, before: MultihopStateV1.on, after: newValue),
                    Change(path: .automatic),
                ]
            )
        }

        let scenario6B = SettingsRule<MultihopStateV2, MultihopSuggestedAction>(
            name: "Scenario 6B"
        ) { input in

            input.tunnelMultihopState == .always && input.daita.isEnabled && !input.daita.directOnlyState.isEnabled
                && input.relayConstraints.entryFilter != .any
        } transform: { input in
            let newValue: MultihopStateV2 = .always

            // Find compatible relays within the country before enabling changing mode to avoid blocking the connection.
            input.tunnelMultihopState = .whenNeeded
            let suggestedLocations = try? relaySelector.selectRelays(tunnelSettings: input, connectionAttemptCount: 0)
            if let suggestedEntry = suggestedLocations?.entry {
                let entryConstraints = UserSelectedRelays(locations: [.country(suggestedEntry.location.countryCode)])
                input.relayConstraints.entryLocations = .only(entryConstraints)
            }

            input.tunnelMultihopState = newValue

            return MigrationResult(
                changes: [
                    Change(path: .updatedMultiHop, before: MultihopStateV1.on, after: newValue),
                    Change(path: .uniqueFilter),
                ],
                action: SuggestedAction(
                    kind: MultihopSuggestedAction.automaticEntry,
                    action: { latestTunnelSettings in
                        latestTunnelSettings.relayConstraints.entryLocations = .any
                    }))
        }
        let scenario7A = SettingsRule<MultihopStateV2, MultihopSuggestedAction>(
            name: "Scenario 7A"
        ) { input in

            input.tunnelMultihopState == .always && input.daita.isEnabled && input.daita.directOnlyState.isEnabled
                && input.relayConstraints.entryFilter == .any
        } transform: { input in

            let newValue: MultihopStateV2 = .always
            input.tunnelMultihopState = newValue

            return MigrationResult(
                changes: [
                    Change(path: .updatedMultiHop, before: MultihopStateV1.on, after: newValue),
                    Change(path: .directOnlyRemoved),
                ])
        }

        let scenario7B = SettingsRule<MultihopStateV2, MultihopSuggestedAction>(
            name: "Scenario 7B"
        ) { input in

            input.tunnelMultihopState == .always && input.daita.isEnabled && input.daita.directOnlyState.isEnabled
                && input.relayConstraints.entryFilter != .any
        } transform: { input in

            let newValue: MultihopStateV2 = .always
            input.tunnelMultihopState = newValue

            return MigrationResult(
                changes: [
                    Change(path: .updatedMultiHop, before: MultihopStateV1.on, after: newValue),
                    Change(path: .directOnlyRemoved),
                    Change(path: .uniqueFilter),
                ],
                action: SuggestedAction(
                    kind: MultihopSuggestedAction.automaticEntry,
                    action: { latestTunnelSettings in
                        latestTunnelSettings.relayConstraints.entryLocations = .any
                    }))
        }

        return SettingsMigrationTracker(rules: [
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
