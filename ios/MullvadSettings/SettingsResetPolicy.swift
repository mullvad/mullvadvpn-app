//
//  SettingsResetPolicy.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-02-25.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

public enum SettingsResetPolicy: Sendable {
    case all
    case none
    case only(Set<SettingsKey>)
    case allExcept(Set<SettingsKey>)

    var keys: Set<SettingsKey> {
        switch self {
        case .all:
            Set(SettingsKey.allCases)
        case .none:
            []
        case .only(let keys):
            keys
        case .allExcept(let excluded):
            Set(SettingsKey.allCases).subtracting(excluded)
        }
    }

    public static var `partially`: SettingsResetPolicy {
        .only([
            .settings,
            .deviceState,
            .apiAccessMethods,
            .ipOverrides,
            .customRelayLists,
        ])
    }
}
