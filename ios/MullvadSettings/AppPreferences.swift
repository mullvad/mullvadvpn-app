//
//  AppPreferences.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-08-09.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation

public protocol AppPreferencesDataSource {
    var hasDoneFirstTimeLaunch: Bool { get set }
    var hasDoneFirstTimeLogin: Bool { get set }
    var isShownOnboarding: Bool { get set }
    var isAgreedToTermsOfService: Bool { get set }
    var lastSeenChangeLogVersion: String { get set }
    var lastVersionCheck: VersionCheck { get set }
    var isNotificationPermissionNeeded: Bool { get set }
    var notificationSettings: NotificationSettings { get set }
}

enum AppStorageKey: String {
    case hasDoneFirstTimeLaunch = "hasFinishedFirstTimeLaunch"
    case hasDoneFirstTimeLogin
    case isShownOnboarding
    case isAgreedToTermsOfService
    case lastSeenChangeLogVersion
    case lastVersionCheck
    case isNotificationPermissionNeeded
    case notificationSettings
}

public final class AppPreferences: AppPreferencesDataSource {
    public init() {}

    @AppStorage(key: AppStorageKey.hasDoneFirstTimeLaunch.rawValue, container: .standard)
    public var hasDoneFirstTimeLaunch: Bool = false

    @AppStorage(key: AppStorageKey.hasDoneFirstTimeLogin.rawValue, container: .standard)
    public var hasDoneFirstTimeLogin: Bool = false

    @AppStorage(key: AppStorageKey.isShownOnboarding.rawValue, container: .standard)
    public var isShownOnboarding = true

    @AppStorage(key: AppStorageKey.isAgreedToTermsOfService.rawValue, container: .standard)
    public var isAgreedToTermsOfService = false

    @AppStorage(key: AppStorageKey.lastSeenChangeLogVersion.rawValue, container: .standard)
    public var lastSeenChangeLogVersion = ""

    @AppStorage(key: AppStorageKey.lastVersionCheck.rawValue, container: .standard)
    public var lastVersionCheck = VersionCheck(version: "", date: .distantPast)

    @AppStorage(key: AppStorageKey.isNotificationPermissionNeeded.rawValue, container: .standard)
    public var isNotificationPermissionNeeded = true

    @AppStorage(key: AppStorageKey.notificationSettings.rawValue, container: .standard)
    public var notificationSettings = NotificationSettings()
}
