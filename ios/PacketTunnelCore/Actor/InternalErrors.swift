//
//  InternalErrors.swift
//  PacketTunnelCore
//
//  Created by pronebird on 14/09/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

/**
 Internal error that's thrown when device state is either `.revoked` or `.loggedOut`.
 */
enum InvalidDeviceStateError: LocalizedError {
    case loggedOut, revoked

    var errorDescription: String? {
        switch self {
        case .loggedOut:
            return "Device is logged out."
        case .revoked:
            return "Device is revoked."
        }
    }
}
