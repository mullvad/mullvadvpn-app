//
//  DefaultPathObserverProtocol.swift
//  PacketTunnelCore
//
//  Created by pronebird on 10/08/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import NetworkExtension

public protocol DefaultPathObserverProtocol {
    /// Returns current default path or `nil` if unknown yet.
    var defaultPath: NetworkPath? { get }

    /// Start observing changes to `defaultPath`.
    func start(_ body: @escaping (NetworkPath) -> Void)

    /// Stop observing changes to `defaultPath`.
    func stop()
}

public protocol NetworkPath {
    var status: NetworkExtension.NWPathStatus { get }
}
