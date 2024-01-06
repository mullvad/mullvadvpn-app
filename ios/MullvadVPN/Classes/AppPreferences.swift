//
//  AppPreferences.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-08-09.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings

protocol AppPreferencesDataSource {
    var isShownOnboarding: Bool { get set }
    var isAgreedToTermsOfService: Bool { get set }
    var lastSeenChangeLogVersion: String { get set }
}

enum AppStorageKey: String {
    case isShownOnboarding, isAgreedToTermsOfService, lastSeenChangeLogVersion
}

final class AppPreferences: AppPreferencesDataSource {
    @AppStorage(key: AppStorageKey.isShownOnboarding.rawValue)
    var isShownOnboarding = true

    @AppStorage(key: AppStorageKey.isAgreedToTermsOfService.rawValue)
    var isAgreedToTermsOfService = false

    @AppStorage(key: AppStorageKey.lastSeenChangeLogVersion.rawValue)
    var lastSeenChangeLogVersion = ""
}
