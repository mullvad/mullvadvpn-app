//
//  AssociatedValue.swift
//  MullvadVPN
//
//  Created by pronebird on 06/07/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// A container type for storing associated values
final class AssociatedValue<T>: NSObject {
    let value: T
    init(_ value: T) {
        self.value = value
    }

    class func get(object: Any, key: UnsafeRawPointer) -> T? {
        let container = objc_getAssociatedObject(object, key) as? Self
        return container?.value
    }

    class func set(object: Any, key: UnsafeRawPointer, value: T?) {
        objc_setAssociatedObject(
            object,
            key,
            value.flatMap { AssociatedValue($0) },
            .OBJC_ASSOCIATION_RETAIN_NONATOMIC
        )
    }
}
