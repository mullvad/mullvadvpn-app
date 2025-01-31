//
//  DefaultPathObserverFake.swift
//  PacketTunnelCoreTests
//
//  Created by pronebird on 16/08/2023.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Network
import NetworkExtension
import PacketTunnelCore

/// Default path observer fake that uses in-memory storage to keep current path and provides a method to simulate path change from tests.
class DefaultPathObserverFake: DefaultPathObserverProtocol, @unchecked Sendable {
    var currentPathStatus: Network.NWPath.Status { .satisfied }
    private var defaultPathHandler: ((Network.NWPath.Status) -> Void)?

    public var onStart: (() -> Void)?
    public var onStop: (() -> Void)?

    func start(_ body: @escaping (Network.NWPath.Status) -> Void) {
        defaultPathHandler = body
        onStart?()
    }

    func stop() {
        defaultPathHandler = nil
        onStop?()
    }

    /// Simulate network path update.
    func updatePath(_ newPath: Network.NWPath.Status) {}
}
