//
//  AppPreferences.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-08-09.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

protocol AppPreferencesDataSource {
    var isShownOnboarding: Bool { set get }
    var isAgreedToTermsOfService: Bool { get set }
    var lastSeenChangeLogVersion: String { get set }
}

@propertyWrapper
struct AppStorage<Value> {
    let key: String
    let defaultValue: Value
    let container: UserDefaults

    var wrappedValue: Value {
        get {
            let value = container.value(forKey: key)
            return value.flatMap { $0 as? Value } ?? defaultValue
        }
        set {
            if let anyOptional = newValue as? AnyOptional,
               anyOptional.isNil {
                container.removeObject(forKey: key)
            } else {
                container.set(newValue, forKey: key)
            }
        }
    }

    init(key: String, defaultValue: Value, container: UserDefaults = .standard) {
        self.key = key
        self.defaultValue = defaultValue
        self.container = container
    }
}

protocol AnyOptional {
    var isNil: Bool { get }
}

extension Optional: AnyOptional {
    var isNil: Bool { self == nil }
}

final class AppPreferences: AppPreferencesDataSource {
    @AppStorage(key: "isShownOnboarding", defaultValue: true)
    var isShownOnboarding: Bool

    @AppStorage(key: "isAgreedToTermsOfService", defaultValue: false)
    var isAgreedToTermsOfService: Bool

    @AppStorage(key: "lastSeenChangeLogVersion", defaultValue: "")
    var lastSeenChangeLogVersion: String
}
