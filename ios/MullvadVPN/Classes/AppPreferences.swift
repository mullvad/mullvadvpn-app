//
//  AppPreferences.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-08-09.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

protocol AppPreferencesDataSource {
    var isShownOnboarding: Bool { get set }
    var isAgreedToTermsOfService: Bool { get set }
    var lastSeenChangeLogVersion: String { get set }
}

enum AppStorageKey: String {
    case isShownOnboarding, isAgreedToTermsOfService, lastSeenChangeLogVersion
}

@propertyWrapper
struct AppStorage<Value> {
    let key: AppStorageKey
    let defaultValue: Value
    let container: UserDefaults

    var wrappedValue: Value {
        get {
            let value = container.value(forKey: key.rawValue)
            return value.flatMap { $0 as? Value } ?? defaultValue
        }
        set {
            if let anyOptional = newValue as? AnyOptional,
               anyOptional.isNil {
                container.removeObject(forKey: key.rawValue)
            } else {
                container.set(newValue, forKey: key.rawValue)
            }
        }
    }

    init(wrappedValue: Value, _ key: AppStorageKey, container: UserDefaults = .standard) {
        self.defaultValue = wrappedValue
        self.container = container
        self.key = key
    }
}

final class AppPreferences: AppPreferencesDataSource {
    @AppStorage(.isShownOnboarding)
    var isShownOnboarding = true

    @AppStorage(.isAgreedToTermsOfService)
    var isAgreedToTermsOfService = false

    @AppStorage(.lastSeenChangeLogVersion)
    var lastSeenChangeLogVersion = ""
}

protocol AnyOptional {
    var isNil: Bool { get }
}

extension Optional: AnyOptional {
    var isNil: Bool { self == nil }
}
