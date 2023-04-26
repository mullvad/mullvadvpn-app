//
//  PacketTunnelStatus.swift
//  MullvadTypes
//
//  Created by pronebird on 27/07/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

public struct DeviceCheck: Codable, Equatable {
    /// Last known account expiry.
    /// Set to `nil` when account expiry is unknown yet.
    public var accountExpiry: Date?

    /// Invalid account. Often happens when account is removed on our backend.
    public var isInvalidAccount: Bool?

    /// Device is revoked.
    public var isRevokedDevice: Bool?

    /// Whether the key stored on device does not match the key stored on backend.
    public var isKeyMismatch: Bool?

    /// Last time packet tunnel had an attempt to rotate the key whether successfully or not.
    public var lastKeyRotationAttemptDate: Date?

    public init(
        accountExpiry: Date? = nil,
        isInvalidAccount: Bool? = nil,
        isRevokedDevice: Bool? = nil,
        isKeyMismatch: Bool? = nil,
        lastKeyRotationDate: Date? = nil
    ) {
        self.accountExpiry = accountExpiry
        self.isInvalidAccount = isInvalidAccount
        self.isRevokedDevice = isRevokedDevice
        self.isKeyMismatch = isKeyMismatch
        lastKeyRotationAttemptDate = lastKeyRotationDate
    }

    public func merged(with other: DeviceCheck) -> DeviceCheck {
        var copyOfSelf = self
        copyOfSelf.merge(with: other)
        return copyOfSelf
    }

    public mutating func merge(with other: DeviceCheck) {
        other.accountExpiry.flatMap { accountExpiry = $0 }
        other.isInvalidAccount.flatMap { isInvalidAccount = $0 }
        other.isRevokedDevice.flatMap { isRevokedDevice = $0 }
        other.isKeyMismatch.flatMap { isKeyMismatch = $0 }
        other.lastKeyRotationAttemptDate.flatMap { lastKeyRotationAttemptDate = $0 }
    }
}

/// Struct describing packet tunnel process status.
public struct PacketTunnelStatus: Codable, Equatable {
    /// Last tunnel error.
    public var lastErrors: [PacketTunnelErrorWrapper]

    /// Flag indicating whether network is reachable.
    public var isNetworkReachable: Bool

    /// Last performed device check.
    public var deviceCheck: DeviceCheck?

    /// Current relay.
    public var tunnelRelay: PacketTunnelRelay?

    /// Number of consecutive connection failure attempts.
    public var numberOfFailedAttempts: UInt

    public init(
        lastErrors: [PacketTunnelErrorWrapper] = [],
        isNetworkReachable: Bool = true,
        deviceCheck: DeviceCheck? = nil,
        tunnelRelay: PacketTunnelRelay? = nil,
        numberOfFailedAttempts: UInt = 0
    ) {
        self.lastErrors = lastErrors
        self.isNetworkReachable = isNetworkReachable
        self.deviceCheck = deviceCheck
        self.tunnelRelay = tunnelRelay
        self.numberOfFailedAttempts = numberOfFailedAttempts
    }
}
