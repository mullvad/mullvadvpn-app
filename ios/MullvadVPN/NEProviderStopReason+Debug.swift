//
//  NEProviderStopReason+Debug.swift
//  PacketTunnel
//
//  Created by pronebird on 14/04/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation
import NetworkExtension

extension NEProviderStopReason: CustomDebugStringConvertible {
    public var debugDescription: String {
        var output = "NEProviderStopReason."
        switch self {
        case .none:
            output += "none"
        case .userInitiated:
            output += "userInitiated"
        case .providerFailed:
            output += "providerFailed"
        case .noNetworkAvailable:
            output += "noNetworkAvailable"
        case .unrecoverableNetworkChange:
            output += "unrecoverableNetworkChange"
        case .providerDisabled:
            output += "providerDisabled"
        case .authenticationCanceled:
            output += "authenticationCanceled"
        case .configurationFailed:
            output += "configurationFailed"
        case .idleTimeout:
            output += "idleTimeout"
        case .configurationDisabled:
            output += "configurationDisabled"
        case .configurationRemoved:
            output += "configurationRemoved"
        case .superceded:
            output += "superceded"
        case .userLogout:
            output += "userLogout"
        case .userSwitch:
            output += "userSwitch"
        case .connectionFailed:
            output += "connectionFailed"
        case .sleep:
            output += "sleep"
        case .appUpdate:
            output += "appUpdate"
        @unknown default:
            output += "\(self.rawValue)"
        }
        return output
    }
}

