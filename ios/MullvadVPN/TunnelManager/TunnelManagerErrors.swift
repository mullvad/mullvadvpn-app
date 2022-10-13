//
//  TunnelManagerErrors.swift
//  MullvadVPN
//
//  Created by pronebird on 07/09/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

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

struct StartTunnelError: LocalizedError, WrappingError {
    private let _underlyingError: Error

    var errorDescription: String? {
        return NSLocalizedString(
            "START_TUNNEL_ERROR",
            tableName: "TunnelManager",
            value: "Failed to start the tunnel.",
            comment: ""
        )
    }

    var underlyingError: Error? {
        return _underlyingError
    }

    init(underlyingError: Error) {
        _underlyingError = underlyingError
    }
}

struct StopTunnelError: LocalizedError, WrappingError {
    private let _underlyingError: Error

    var errorDescription: String? {
        return NSLocalizedString(
            "STOP_TUNNEL_ERROR",
            tableName: "TunnelManager",
            value: "Failed to stop the tunnel.",
            comment: ""
        )
    }

    var underlyingError: Error? {
        return _underlyingError
    }

    init(underlyingError: Error) {
        _underlyingError = underlyingError
    }
}
