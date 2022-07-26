//
//  TunnelManagerErrors.swift
//  MullvadVPN
//
//  Created by pronebird on 07/09/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import NetworkExtension

struct UnsetTunnelError: LocalizedError {
    var errorDescription: String? {
        return NSLocalizedString(
            "UNSET_TUNNEL_ERROR",
            tableName: "TunnelManager",
            value: "Tunnel is unset.",
            comment: ""
        )
    }
}

struct InvalidDeviceStateError: LocalizedError {
    var errorDescription: String? {
        return NSLocalizedString(
            "INVALID_DEVICE_STATE_ERROR",
            tableName: "TunnelManager",
            value: "Invalid device state.",
            comment: ""
        )
    }
}

struct StartTunnelError: LocalizedError {
    var errorDescription: String? {
        return NSLocalizedString(
            "START_TUNNEL_ERROR",
            tableName: "TunnelManager",
            value: "Failed to start the tunnel.",
            comment: ""
        )
    }

    let underlyingError: Error
    init(underlyingError: Error) {
        self.underlyingError = underlyingError
    }
}

struct StopTunnelError: LocalizedError {
    var errorDescription: String? {
        return NSLocalizedString(
            "STOP_TUNNEL_ERROR",
            tableName: "TunnelManager",
            value: "Failed to stop the tunnel.",
            comment: ""
        )
    }

    let underlyingError: Error
    init(underlyingError: Error) {
        self.underlyingError = underlyingError
    }
}
