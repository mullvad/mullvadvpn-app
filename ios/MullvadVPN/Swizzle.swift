//
//  Swizzle.swift
//  MullvadVPN
//
//  Created by pronebird on 28/10/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation

@inlinable func swizzleMethod(aClass: AnyClass, originalSelector: Selector, newSelector: Selector) {
    guard let originalMethod = class_getInstanceMethod(aClass, originalSelector),
          let newMethod = class_getInstanceMethod(aClass, newSelector) else { return }

    if class_addMethod(
        aClass,
        originalSelector,
        method_getImplementation(newMethod),
        method_getTypeEncoding(newMethod)
    ) {
        class_replaceMethod(
            aClass,
            newSelector,
            method_getImplementation(originalMethod),
            method_getTypeEncoding(originalMethod)
        )
    } else {
        method_exchangeImplementations(originalMethod, newMethod)
    }
}
