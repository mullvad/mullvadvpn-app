//
//  PacketTunnelStatus.swift
//  MullvadTypes
//
//  Created by pronebird on 27/07/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

public enum AccountStateVerdict: Equatable, Codable {
    case failed
    case invalidAccount
    case succeeded(_ accountExpiry: Date)
}

public enum DeviceStateVerdict: Equatable, Codable {
    case failed
    case revoked
    case keyMismatch
    case succeeded
}

public struct DeviceCheck: Codable, Equatable {
    public var accountVerdict: AccountStateVerdict
    public var deviceVerdict: DeviceStateVerdict

    /// Last time packet tunnel had an attempt to rotate the key whether successfully or not.
    public var lastKeyRotationAttemptDate: Date?

    public init(
        accountVerdict: AccountStateVerdict,
        deviceVerdict: DeviceStateVerdict,
        lastKeyRotationAttemptDate: Date?
    ) {
        self.accountVerdict = accountVerdict
        self.deviceVerdict = deviceVerdict
        self.lastKeyRotationAttemptDate = lastKeyRotationAttemptDate
    }

    public func merged(with other: DeviceCheck) -> DeviceCheck {
        var copyOfSelf = self
        copyOfSelf.merge(with: other)
        return copyOfSelf
    }

    public mutating func merge(with other: DeviceCheck) {
        if other.accountVerdict != .failed {
            accountVerdict = other.accountVerdict
        }

        if other.deviceVerdict != .failed {
            deviceVerdict = other.deviceVerdict
        }

        if let lastKeyRotationAttemptDate = other.lastKeyRotationAttemptDate {
            self.lastKeyRotationAttemptDate = lastKeyRotationAttemptDate
        }
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
