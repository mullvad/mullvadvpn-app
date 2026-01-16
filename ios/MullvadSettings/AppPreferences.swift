//
//  AppPreferences.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-08-09.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation

public protocol AppPreferencesDataSource {
    var hasDoneFirstTimeLaunch: Bool { get set }
    var hasDoneFirstTimeLogin: Bool { get set }
    var isShownOnboarding: Bool { get set }
    var isAgreedToTermsOfService: Bool { get set }
    var lastSeenChangeLogVersion: String { get set }
    var lastVersionCheckVersion: String { get set }
    var lastVersionCheckDate: Date { get set }
}

enum AppStorageKey: String {
    case hasDoneFirstTimeLaunch = "hasFinishedFirstTimeLaunch"
    case hasDoneFirstTimeLogin
    case isShownOnboarding
    case isAgreedToTermsOfService
    case lastSeenChangeLogVersion
    case lastVersionCheckVersion
    case lastVersionCheckDate
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

    @AppStorage(key: AppStorageKey.lastVersionCheckVersion.rawValue, container: .standard)
    public var lastVersionCheckVersion = ""

    @AppStorage(key: AppStorageKey.lastVersionCheckDate.rawValue, container: .standard)
    public var lastVersionCheckDate = Date.distantPast
}
