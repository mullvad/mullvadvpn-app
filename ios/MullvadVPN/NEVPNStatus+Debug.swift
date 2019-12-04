//
//  NEVPNStatus+Debug.swift
//  MullvadVPN
//
//  Created by pronebird on 28/11/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import Foundation
import NetworkExtension

extension NEVPNStatus: CustomDebugStringConvertible {
    public var debugDescription: String {
        var output = "NEVPNStatus."
        switch self {
        case .connected:
            output += "connected"
        case .connecting:
            output +=  "connecting"
        case .disconnected:
            output +=  "disconnected"
        case .disconnecting:
            output +=  "disconnecting"
        case .invalid:
            output +=  "invalid"
        case .reasserting:
            output +=  "reasserting"
        @unknown default:
            output += "\(self.rawValue)"
        }
        return output
    }
}
