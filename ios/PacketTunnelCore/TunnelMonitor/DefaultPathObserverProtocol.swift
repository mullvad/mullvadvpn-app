//
//  DefaultPathObserverProtocol.swift
//  PacketTunnelCore
//
//  Created by pronebird on 10/08/2023.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Network

/// A type providing default path access and observation.
public protocol DefaultPathObserverProtocol: Sendable {
    /// Returns current default path or `nil` if unknown yet.
    var currentPathStatus: Network.NWPath.Status { get }

    /// Start observing changes to `defaultPath`.
    /// This call must be idempotent. Multiple calls to start should replace the existing handler block.
    func start(_ body: @escaping @Sendable (Network.NWPath.Status) -> Void)

    /// Stop observing changes to `defaultPath`.
    func stop()
}
