//
//  MigratedSettingsState.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-06-08.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

public struct MigratedSettingsState: Codable {
    public var preMigrationSettings: LatestTunnelSettings?
    public var lastInstalledVersion: String
    public var lastMigratedVersion: Int
    public var hasCompletedMigrationWizard: Bool
    public var shouldShowMigratedSettingsMenuItem: Bool

    public init(
        preMigrationSettings: LatestTunnelSettings? = nil, lastInstalledVersion: String, lastMigratedVersion: Int,
        hasCompletedMigrationWizard: Bool, shouldShowMigratedSettingsMenuItem: Bool
    ) {
        self.preMigrationSettings = preMigrationSettings
        self.lastInstalledVersion = lastInstalledVersion
        self.lastMigratedVersion = lastMigratedVersion
        self.hasCompletedMigrationWizard = hasCompletedMigrationWizard
        self.shouldShowMigratedSettingsMenuItem = shouldShowMigratedSettingsMenuItem
    }
}
