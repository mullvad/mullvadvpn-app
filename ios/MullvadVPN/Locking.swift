//
//  Locking.swift
//  MullvadVPN
//
//  Created by pronebird on 04/02/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension NSLock {
    func withCriticalBlock<T>(_ body: () -> T) -> T {
        lock()
        defer { unlock() }

        return body()
    }
}

extension NSRecursiveLock {
    func withCriticalBlock<T>(_ body: () -> T) -> T {
        lock()
        defer { unlock() }

        return body()
    }
}
