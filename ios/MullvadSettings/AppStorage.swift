//
//  AppStorage.swift
//  MullvadSettings
//
//  Created by Mojgan on 2024-01-05.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging

@propertyWrapper
public struct PrimitiveStorage<Value: UserDefaultsPrimitive> {
    let key: String
    let defaultValue: Value
    let container: UserDefaults

    public var wrappedValue: Value {
        get {
            container.value(forKey: key) as? Value ?? defaultValue
        }
        set {
            if let anyOptional = newValue as? AnyOptional,
                anyOptional.isNil
            {
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

@propertyWrapper
public struct CompositeStorage<Value: Codable> {
    let logger = Logger(label: "CompositeStorage")
    private let key: String
    private let defaultValue: Value
    private let container: UserDefaults

    public var wrappedValue: Value {
        get {
            guard let data = container.data(forKey: key),
                let decoded = try? JSONDecoder().decode(Value.self, from: data)
            else {
                return container.value(forKey: key) as? Value ?? defaultValue
            }
            return decoded
        }
        set {
            do {
                let data = try JSONEncoder().encode(newValue)
                container.set(data, forKey: key)
            } catch {
                logger.error("Failed to encode \(Value.self)")
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

public protocol UserDefaultsPrimitive {}
extension String: UserDefaultsPrimitive {}
extension Int: UserDefaultsPrimitive {}
extension Bool: UserDefaultsPrimitive {}
extension Double: UserDefaultsPrimitive {}
extension Data: UserDefaultsPrimitive {}
extension Array: UserDefaultsPrimitive where Element: UserDefaultsPrimitive {}
extension Dictionary: UserDefaultsPrimitive where Key == String, Value: UserDefaultsPrimitive {}
