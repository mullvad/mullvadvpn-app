// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

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
