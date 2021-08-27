//
//  Promise+Optional.swift
//  Promise+Optional
//
//  Created by pronebird on 22/08/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension Optional {
    func asPromise() -> Promise<Self> {
        return .resolved(self)
    }
}

extension Promise where Value: AnyOptional {
    /// Map the value when present. Returns `defaultValue` otherwise.
    func map<NewValue>(defaultValue: NewValue, transform: @escaping (Value.Wrapped) -> NewValue) -> Promise<NewValue> {
        return then { value -> NewValue in
            switch value.asConcreteType() {
            case .some(let unwrappedValue):
                return transform(unwrappedValue)
            case .none:
                return defaultValue
            }
        }
    }

    /// Map the value when present, producing new promise to compute the new value. Returns `defaultValue` otherwise.
    func mapThen<NewValue>(defaultValue: NewValue, producePromise: @escaping (Value.Wrapped) -> Promise<NewValue>) -> Promise<NewValue> {
        return then { value in
            switch value.asConcreteType() {
            case .some(let unwrappedValue):
                return producePromise(unwrappedValue)
            case .none:
                return .resolved(defaultValue)
            }
        }
    }
}
