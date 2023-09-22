//
//  DefaultPathObserverProtocol.swift
//  PacketTunnelCore
//
//  Created by pronebird on 10/08/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import NetworkExtension

/// A type providing default path access and observation.
public protocol DefaultPathObserverProtocol {
    /// Returns current default path or `nil` if unknown yet.
    var defaultPath: NetworkPath? { get }

    /// Start observing changes to `defaultPath`.
    /// This call must be idempotent. Multiple calls to start should replace the existing handler block.
    func start(_ body: @escaping (NetworkPath) -> Void)

    /// Stop observing changes to `defaultPath`.
    func stop()
}

/// A type that represents a network path.
public protocol NetworkPath {
    var status: NetworkExtension.NWPathStatus { get }
}
