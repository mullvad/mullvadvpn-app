// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation

public protocol AppPreferencesDataSource {
    var hasDoneFirstTimeLaunch: Bool { get set }
    var hasDoneFirstTimeLogin: Bool { get set }
    var isShownOnboarding: Bool { get set }
    var isAgreedToTermsOfService: Bool { get set }
    var lastSeenChangeLogVersion: String { get set }
    var lastVersionCheck: VersionCheck { get set }
    var isNotificationPermissionAsked: Bool { get set }
    var notificationSettings: NotificationSettings { get set }
    var includeAllNetworksConsent: Bool { get set }
    var migratedSettingsState: MigratedSettingsState { get set }
}

enum AppStorageKey: String {
    case hasDoneFirstTimeLaunch = "hasFinishedFirstTimeLaunch"
    case hasDoneFirstTimeLogin
    case isShownOnboarding
    case isAgreedToTermsOfService
    case lastSeenChangeLogVersion
    case lastVersionCheck
    case isNotificationPermissionAsked
    case notificationSettings
    case includeAllNetworksConsent
    case migratedSettingsState
}

public final class AppPreferences: AppPreferencesDataSource {
    @PrimitiveStorage(key: AppStorageKey.hasDoneFirstTimeLaunch.rawValue, container: .standard)
    public var hasDoneFirstTimeLaunch: Bool = false

    @PrimitiveStorage(key: AppStorageKey.hasDoneFirstTimeLogin.rawValue, container: .standard)
    public var hasDoneFirstTimeLogin: Bool = false

    @PrimitiveStorage(key: AppStorageKey.isShownOnboarding.rawValue, container: .standard)
    public var isShownOnboarding = true

    @PrimitiveStorage(key: AppStorageKey.isAgreedToTermsOfService.rawValue, container: .standard)
    public var isAgreedToTermsOfService = false

    @PrimitiveStorage(key: AppStorageKey.lastSeenChangeLogVersion.rawValue, container: .standard)
    public var lastSeenChangeLogVersion = ""

    @CompositeStorage(key: AppStorageKey.lastVersionCheck.rawValue, container: .standard)
    public var lastVersionCheck = VersionCheck(version: "", date: .distantPast)

    @PrimitiveStorage(key: AppStorageKey.isNotificationPermissionAsked.rawValue, container: .standard)
    public var isNotificationPermissionAsked = false

    @CompositeStorage(key: AppStorageKey.notificationSettings.rawValue, container: .standard)
    public var notificationSettings = NotificationSettings()

    @PrimitiveStorage(key: AppStorageKey.includeAllNetworksConsent.rawValue, container: .standard)
    public var includeAllNetworksConsent = false

    @CompositeStorage(key: AppStorageKey.migratedSettingsState.rawValue, container: .standard)
    public var migratedSettingsState = MigratedSettingsState(
        preMigrationSettings: nil,
        lastInstalledVersion: "",
        lastMigratedVersion: MigratedVersion.v1.rawValue,
        hasCompletedMigrationWizard: true,
        shouldShowMigratedSettingsMenuItem: false)

    public init() {
        migratedSettingsState.lastInstalledVersion =
            migratedSettingsState.lastInstalledVersion.isEmpty
            ? lastSeenChangeLogVersion : migratedSettingsState.lastInstalledVersion
    }
}
