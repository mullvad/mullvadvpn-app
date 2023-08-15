//
//  MockDefaultPathObserver.swift
//  PacketTunnelCoreTests
//
//  Created by pronebird on 16/08/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import NetworkExtension
import PacketTunnelCore

struct MockNetworkPath: NetworkPath {
    var status: NetworkExtension.NWPathStatus = .satisfied
}

/// Mock implementation of a default path observer.
class MockDefaultPathObserver: DefaultPathObserverProtocol {
    var defaultPath: NetworkPath? {
        return stateLock.withLock { innerPath }
    }

    private var innerPath: NetworkPath = MockNetworkPath()
    private var stateLock = NSLock()

    private var defaultPathHandler: ((NetworkPath) -> Void)?

    func start(_ body: @escaping (NetworkPath) -> Void) {
        stateLock.withLock {
            defaultPathHandler = body
        }
    }

    func stop() {
        stateLock.withLock {
            defaultPathHandler = nil
        }
    }

    /// Simulate network path update.
    func updatePath(_ newPath: NetworkPath) {
        let pathHandler = stateLock.withLock {
            innerPath = newPath
            return defaultPathHandler
        }
        pathHandler?(newPath)
    }
}
