// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation

public struct WeakBox<T>: Sendable {
    public var value: T? {
        valueProvider()
    }

    nonisolated(unsafe) private let valueProvider: () -> T?

    public init(_ value: T) {
        let reference = value as AnyObject

        valueProvider = { [weak reference] in
            reference as? T
        }
    }

    static func == (lhs: WeakBox<T>, rhs: WeakBox<T>) -> Bool {
        (lhs.value as AnyObject) === (rhs.value as AnyObject)
    }
}

final public class ObserverList<T>: Sendable {
    private let lock = NSLock()
    nonisolated(unsafe) private var observers = [WeakBox<T>]()

    public init() {}

    public func append(_ observer: T) {
        lock.withLock {
            let hasObserver = observers.contains { box in
                box == WeakBox(observer)
            }

            if !hasObserver {
                observers.append(WeakBox(observer))
            }
        }
    }

    public func remove(_ observer: T) {
        lock.withLock {
            let index = observers.firstIndex { box in
                box == WeakBox(observer)
            }

            if let index {
                observers.remove(at: index)
            }
        }
    }

    public func notify(_ body: (T) -> Void) {
        var indicesToRemove = [Int]()
        var observersToNotify = [T]()

        lock.withLock {
            for (index, box) in observers.enumerated() {
                if let observer = box.value {
                    observersToNotify.append(observer)
                } else {
                    indicesToRemove.append(index)
                }
            }

            for index in indicesToRemove.reversed() {
                observers.remove(at: index)
            }
        }

        for observer in observersToNotify {
            body(observer)
        }
    }
}
