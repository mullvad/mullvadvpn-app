//
//  DeviceCheck.swift
//  PacketTunnel
//
//  Created by pronebird on 13/09/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// The verdict of an account status check.
public enum AccountVerdict: Equatable {
    /// Account is no longer valid.
    case invalid

    /// Account is expired.
    case expired(Account)

    /// Account exists and has enough time left.
    case active(Account)
}

/// The verdict of a device status check.
public enum DeviceVerdict: Equatable {
    /// Device is revoked.
    case revoked

    /// Device exists but the public key registered on server does not match any longer.
    case keyMismatch

    /// Device is in good standing and should work as normal.
    case active
}

/// Type describing whether key rotation took place and the outcome of it.
public enum KeyRotationStatus: Equatable {
    /// No rotation took place yet.
    case noAction

    /// Rotation attempt took place but without success.
    case attempted(Date)

    /// Rotation attempt took place and succeeded.
    case succeeded(Date)

    /// Returns `true` if the status is `.succeeded`.
    public var isSucceeded: Bool {
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
public struct DeviceCheck: Equatable {
    /// The verdict of account status check.
    public var accountVerdict: AccountVerdict

    /// The verdict of device status check.
    public var deviceVerdict: DeviceVerdict

    // The status of the last performed key rotation.
    public var keyRotationStatus: KeyRotationStatus

    public init(accountVerdict: AccountVerdict, deviceVerdict: DeviceVerdict, keyRotationStatus: KeyRotationStatus) {
        self.accountVerdict = accountVerdict
        self.deviceVerdict = deviceVerdict
        self.keyRotationStatus = keyRotationStatus
    }
}
