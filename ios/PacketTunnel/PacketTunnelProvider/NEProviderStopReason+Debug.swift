//
//  NEProviderStopReason+Debug.swift
//  PacketTunnel
//
//  Created by pronebird on 14/04/2020.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import NetworkExtension

struct ProviderStopReasonWrapper: CustomStringConvertible {
    let reason: NEProviderStopReason

    public var description: String {
        switch reason {
        case .none:
            return "none"
        case .userInitiated:
            return "user initiated"
        case .providerFailed:
            return "provider failed"
        case .noNetworkAvailable:
            return "no network available"
        case .unrecoverableNetworkChange:
            return "unrecoverable network change"
        case .providerDisabled:
            return "provider disabled"
        case .authenticationCanceled:
            return "authentication cancelled"
        case .configurationFailed:
            return "configuration failed"
        case .idleTimeout:
            return "idle timeout"
        case .configurationDisabled:
            return "configuration disabled"
        case .configurationRemoved:
            return "configuration removed"
        case .superceded:
            return "superceded"
        case .userLogout:
            return "user logout"
        case .userSwitch:
            return "user switch"
        case .connectionFailed:
            return "connection failed"
        case .sleep:
            return "sleep"
        case .appUpdate:
            return "app update"
        case .internalError:
            return "internal error"
        @unknown default:
            return "unknown value (\(reason.rawValue))"
        }
    }
}
