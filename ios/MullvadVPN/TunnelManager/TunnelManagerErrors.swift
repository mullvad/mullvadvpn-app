//
//  TunnelManagerErrors.swift
//  MullvadVPN
//
//  Created by pronebird on 07/09/2021.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

struct UnsetTunnelError: LocalizedError {
    var errorDescription: String? {
        NSLocalizedString(
            "UNSET_TUNNEL_ERROR",
            value: "Tunnel is unset.",
            comment: ""
        )
    }
}

struct InvalidDeviceStateError: LocalizedError {
    var errorDescription: String? {
        NSLocalizedString(
            "INVALID_DEVICE_STATE_ERROR",
            value: "Invalid device state.",
            comment: ""
        )
    }
}

struct StartTunnelError: LocalizedError, WrappingError {
    private let _underlyingError: Error

    var errorDescription: String? {
        NSLocalizedString(
            "START_TUNNEL_ERROR",
            value: "Failed to start the tunnel.",
            comment: ""
        )
    }

    var underlyingError: Error? {
        _underlyingError
    }

    init(underlyingError: Error) {
        _underlyingError = underlyingError
    }
}

struct StopTunnelError: LocalizedError, WrappingError {
    private let _underlyingError: Error

    var errorDescription: String? {
        NSLocalizedString(
            "STOP_TUNNEL_ERROR",
            value: "Failed to stop the tunnel.",
            comment: ""
        )
    }

    var underlyingError: Error? {
        _underlyingError
    }

    init(underlyingError: Error) {
        _underlyingError = underlyingError
    }
}
