// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

public struct MigratedSettingsState: Codable, Sendable {
    public var preMigrationSettings: LatestTunnelSettings?
    public var lastInstalledVersion: String
    public var lastMigratedVersion: Int
    public var hasCompletedMigrationWizard: Bool
    public var shouldShowMigratedSettingsMenuItem: Bool

    public init(
        preMigrationSettings: LatestTunnelSettings?,
        lastInstalledVersion: String,
        lastMigratedVersion: Int,
        hasCompletedMigrationWizard: Bool,
        shouldShowMigratedSettingsMenuItem: Bool
    ) {
        self.preMigrationSettings = preMigrationSettings
        self.lastInstalledVersion = lastInstalledVersion
        self.lastMigratedVersion = lastMigratedVersion
        self.hasCompletedMigrationWizard = hasCompletedMigrationWizard
        self.shouldShowMigratedSettingsMenuItem = shouldShowMigratedSettingsMenuItem
    }
}
