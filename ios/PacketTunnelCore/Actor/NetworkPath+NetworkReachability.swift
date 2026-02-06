//
//  NetworkPath+.swift
//  PacketTunnelCore
//
//  Created by pronebird on 14/09/2023.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Network

extension Network.NWPath.Status {
    /// Converts `NetworkPath.status` into `NetworkReachability`.
    public var networkReachability: NetworkReachability {
        switch self {
        case .satisfied:
            .reachable
        case .unsatisfied:
            .unreachable
        case .requiresConnection:
            .reachable
        @unknown default:
            .undetermined
        }
    }
}
