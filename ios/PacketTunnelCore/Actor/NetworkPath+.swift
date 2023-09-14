//
//  NetworkPath+.swift
//  PacketTunnelCore
//
//  Created by pronebird on 14/09/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension NetworkPath {
    /// Converts `NetworkPath.status` into `NetworkReachability`.
    var networkReachability: NetworkReachability {
        switch status {
        case .satisfiable, .satisfied:
            return .reachable

        case .unsatisfied:
            return .unreachable

        case .invalid:
            return .undetermined

        @unknown default:
            return .undetermined
        }
    }
}
