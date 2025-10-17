//
//  AppPreferences.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-08-09.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings

protocol AppPreferencesDataSource {
    var hasDoneFirstTimeLaunch: Bool { get set }
    var hasDoneFirstTimeLogin: Bool { get set }
    var isShownOnboarding: Bool { get set }
    var isAgreedToTermsOfService: Bool { get set }
    var lastSeenChangeLogVersion: String { get set }
}

enum AppStorageKey: String {
    case hasDoneFirstTimeLaunch = "hasFinishedFirstTimeLaunch"
    case hasDoneFirstTimeLogin
    case isShownOnboarding
    case isAgreedToTermsOfService
    case lastSeenChangeLogVersion
}

final class AppPreferences: AppPreferencesDataSource {
    @AppStorage(key: AppStorageKey.hasDoneFirstTimeLaunch.rawValue, container: .standard)
    var hasDoneFirstTimeLaunch: Bool = false

    @AppStorage(key: AppStorageKey.hasDoneFirstTimeLogin.rawValue, container: .standard)
    var hasDoneFirstTimeLogin: Bool = false

    @AppStorage(key: AppStorageKey.isShownOnboarding.rawValue, container: .standard)
    var isShownOnboarding = true

    @AppStorage(key: AppStorageKey.isAgreedToTermsOfService.rawValue, container: .standard)
    var isAgreedToTermsOfService = false

    @AppStorage(key: AppStorageKey.lastSeenChangeLogVersion.rawValue, container: .standard)
    var lastSeenChangeLogVersion = ""
}
