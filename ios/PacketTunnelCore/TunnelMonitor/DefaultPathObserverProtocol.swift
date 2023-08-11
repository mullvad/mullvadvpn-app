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
    /// Returns cancellation token that will terminate observation either upon deallocation or upon explicit call to `invalidate()`.
    func observe(_ body: @escaping (NetworkPath) -> Void) -> DefaultPathObservation
}

public protocol DefaultPathObservation {
    func invalidate()
}

public protocol NetworkPath {
    var status: NetworkExtension.NWPathStatus { get }
}
