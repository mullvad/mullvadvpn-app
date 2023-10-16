//
//  DefaultPathObserverFake.swift
//  PacketTunnelCoreTests
//
//  Created by pronebird on 16/08/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import NetworkExtension
import PacketTunnelCore

struct NetworkPathStub: NetworkPath {
    var status: NetworkExtension.NWPathStatus = .satisfied
}

/// Default path observer fake that uses in-memory storage to keep current path and provides a method to simulate path change from tests.
class DefaultPathObserverFake: DefaultPathObserverProtocol {
    var defaultPath: NetworkPath? {
        return stateLock.withLock { innerPath }
    }

    private var innerPath: NetworkPath = NetworkPathStub()
    private var stateLock = NSLock()
    private var defaultPathHandler: ((NetworkPath) -> Void)?

    public var onStart: (() -> Void)?
    public var onStop: (() -> Void)?

    func start(_ body: @escaping (NetworkPath) -> Void) {
        stateLock.withLock {
            defaultPathHandler = body
            onStart?()
        }
    }

    func stop() {
        stateLock.withLock {
            defaultPathHandler = nil
            onStop?()
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
