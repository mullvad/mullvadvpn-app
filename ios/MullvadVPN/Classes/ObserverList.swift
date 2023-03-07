//
//  ObserverList.swift
//  MullvadVPN
//
//  Created by pronebird on 26/06/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation

struct WeakBox<T> {
    var value: T? {
        return valueProvider()
    }

    private let valueProvider: () -> T?

    init(_ value: T) {
        let reference = value as AnyObject

        valueProvider = { [weak reference] in
            return reference as? T
        }
    }

    static func == (lhs: WeakBox<T>, rhs: T) -> Bool {
        return (lhs.value as AnyObject) === (rhs as AnyObject)
    }
}

final class ObserverList<T> {
    private let lock = NSLock()
    private var observers = [WeakBox<T>]()

    func append(_ observer: T) {
        lock.lock()

        let hasObserver = observers.contains { box in
            return box == observer
        }

        if !hasObserver {
            observers.append(WeakBox(observer))
        }

        lock.unlock()
    }

    func remove(_ observer: T) {
        lock.lock()

        let index = observers.firstIndex { box in
            return box == observer
        }

        if let index = index {
            observers.remove(at: index)
        }

        lock.unlock()
    }

    func forEach(_ body: (T) -> Void) {
        lock.lock()

        var indicesToRemove = [Int]()
        var observersToNotify = [T]()

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

        lock.unlock()

        for observer in observersToNotify {
            body(observer)
        }
    }
}
