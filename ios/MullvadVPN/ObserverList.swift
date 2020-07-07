//
//  ObserverList.swift
//  MullvadVPN
//
//  Created by pronebird on 26/06/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation

protocol WeakObserverBox: Equatable {
    associatedtype Wrapped

    var inner: Wrapped? { get }
}

class ObserverList<T: WeakObserverBox> {
    private let lock = NSRecursiveLock()
    private var observers = [T]()

    func append(_ observer: T) {
        lock.withCriticalBlock {
            if !self.observers.contains(observer) {
                self.observers.append(observer)
            }
        }
    }

    func remove(_ observer: T) {
        lock.withCriticalBlock {
            self.observers.removeAll { $0 == observer }
        }
    }

    func forEach(_ body: (T) -> Void) {
        lock.withCriticalBlock {
            var discardObservers = [T]()
            self.observers.forEach { (boxedObserver) in
                body(boxedObserver)

                if boxedObserver.inner == nil {
                    discardObservers.append(boxedObserver)
                }
            }

            if !discardObservers.isEmpty {
                self.observers.removeAll { (observer) -> Bool in
                    return discardObservers.contains(observer)
                }
            }
        }
    }
}
