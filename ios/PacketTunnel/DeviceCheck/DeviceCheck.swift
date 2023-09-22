//
//  DeviceCheck.swift
//  PacketTunnel
//
//  Created by pronebird on 13/09/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

/// The verdict of an account status check.
enum AccountVerdict: Equatable {
    /// Account is no longer valid.
    case invalid

    /// Account is expired.
    case expired(Account)

    /// Account exists and has enough time left.
    case active(Account)
}

/// The verdict of a device status check.
enum DeviceVerdict: Equatable {
    /// Device is revoked.
    case revoked

    /// Device exists but the public key registered on server does not match any longer.
    case keyMismatch

    /// Device is in good standing and should work as normal.
    case active
}

/// Type describing whether key rotation took place and the outcome of it.
enum KeyRotationStatus: Equatable {
    /// No rotation took place yet.
    case noAction

    /// Rotation attempt took place but without success.
    case attempted(Date)

    /// Rotation attempt took place and succeeded.
    case succeeded(Date)

    /// Returns `true` if the status is `.succeeded`.
    var isSucceeded: Bool {
        if case .succeeded = self {
            return true
        } else {
            return false
        }
    }
}

/**
 Struct holding data associated with account and device diagnostics and also device key recovery performed by packet
 tunnel process.
 */
struct DeviceCheck: Equatable {
    /// The verdict of account status check.
    var accountVerdict: AccountVerdict

    /// The verdict of device status check.
    var deviceVerdict: DeviceVerdict

    // The status of the last performed key rotation.
    var keyRotationStatus: KeyRotationStatus
}
