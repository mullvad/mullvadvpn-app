//
//  ObserverList.swift
//  MullvadVPN
//
//  Created by pronebird on 26/06/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

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
        lock.lock()

        let hasObserver = observers.contains { box in
            box == WeakBox(observer)
        }

        if !hasObserver {
            observers.append(WeakBox(observer))
        }

        lock.unlock()
    }

    public func remove(_ observer: T) {
        lock.lock()

        let index = observers.firstIndex { box in
            box == WeakBox(observer)
        }

        if let index {
            observers.remove(at: index)
        }

        lock.unlock()
    }

    public func notify(_ body: (T) -> Void) {
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
