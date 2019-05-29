//
//  WeakBox.swift
//  MullvadVPN
//
//  Created by pronebird on 29/05/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import Foundation

final class WeakBox<T: AnyObject> {
    weak var unboxed: T?

    init(_ value: T) {
        unboxed = value
    }
}
