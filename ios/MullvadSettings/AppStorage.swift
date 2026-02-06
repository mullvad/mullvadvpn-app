//
//  AppStorage.swift
//  MullvadSettings
//
//  Created by Mojgan on 2024-01-05.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation

@propertyWrapper
public struct AppStorage<Value: Codable> {
    let key: String
    let defaultValue: Value
    let container: UserDefaults

    public var wrappedValue: Value {
        get {
            guard
                let data = container.data(forKey: key),
                let value = try? JSONDecoder().decode(Value.self, from: data)
            else {
                return defaultValue
            }
            return value
        }
        set {
            if let data = try? JSONEncoder().encode(newValue) {
                container.set(data, forKey: key)
            } else {
                container.removeObject(forKey: key)
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
