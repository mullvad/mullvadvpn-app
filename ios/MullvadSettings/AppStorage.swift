//
//  AppStorage.swift
//  MullvadSettings
//
//  Created by Mojgan on 2024-01-05.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
@propertyWrapper
public struct AppStorage<Value> {
    let key: String
    let defaultValue: Value
    let container: UserDefaults

    public var wrappedValue: Value {
        get {
            container.value(forKey: key) as? Value ?? defaultValue
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

    public init(wrappedValue: Value, key: String, container: UserDefaults) {
        self.defaultValue = wrappedValue
        self.container = container
        self.key = key
    }
}

protocol AnyOptional {
    var isNil: Bool { get }
}

extension Optional: AnyOptional {
    var isNil: Bool { self == nil }
}
