//
//  SettingsMigrationTracker.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-04-01.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadLogging
import MullvadREST
import MullvadSettings
import MullvadTypes

struct SettingsRule<Output: Sendable, ActionKind: Sendable>: Sendable {
    let name: String
    let condition: @Sendable (LatestTunnelSettings) -> Bool

    let transform:
        @Sendable (
            inout LatestTunnelSettings
        ) -> MigrationResult<Output, ActionKind>
}

enum SettingsUpdate: Sendable {
    case none
    case automatic
    case directOnlyRemoved
    case uniqueFilter
    case updatedMultiHop
}

struct Change: @unchecked Sendable {
    let path: SettingsUpdate
    var before: Any? = nil
    var after: Any? = nil
}

struct SuggestedAction<ActionKind: Sendable>: Sendable {
    let kind: ActionKind
    let action: (@Sendable (inout LatestTunnelSettings) -> Void)?
}

struct MigrationResult<Output: Sendable, ActionKind: Sendable>: Sendable {
    let changes: [Change]
    var action: SuggestedAction<ActionKind>? = nil
}

enum MigrationError: Error {
    case noMatchingRule
}

struct SettingsMigrationTracker<Output: Sendable, ActionKind: Sendable>: Sendable {
    let logger = Logger(label: "SettingsMigrationTracker")
    let rules: [SettingsRule<Output, ActionKind>]
    func run(
        input: inout LatestTunnelSettings
    ) throws -> MigrationResult<Output, ActionKind> {

        for rule in rules {
            if rule.condition(input) {
                logger.debug("\(rule.name) is matching")
                return rule.transform(&input)
            }
        }

        throw MigrationError.noMatchingRule
    }
}
