//
//  NEVPNStatus+Debug.swift
//  MullvadVPN
//
//  Created by pronebird on 28/11/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Foundation
import NetworkExtension

extension NEVPNStatus: CustomStringConvertible {
    public var description: String {
        switch self {
        case .connected:
            return "connected"
        case .connecting:
            return "connecting"
        case .disconnected:
            return "disconnected"
        case .disconnecting:
            return "disconnecting"
        case .invalid:
            return "invalid"
        case .reasserting:
            return "reasserting"
        @unknown default:
            return "unknown value (\(rawValue))"
        }
    }
}
