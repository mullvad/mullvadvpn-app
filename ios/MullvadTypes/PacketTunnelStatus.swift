//
//  PacketTunnelStatus.swift
//  MullvadTypes
//
//  Created by pronebird on 27/07/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

public struct DeviceCheck: Codable, Equatable {
    /// Unique identifier for the device check.
    /// Should only change when other fields in the struct are being changed.
    public var identifier: UUID

    /// Flag indicating whether device is revoked.
    /// Set to `nil` when the device status is unknown yet.
    public var isDeviceRevoked: Bool?

    /// Last known account expiry.
    /// Set to `nil` when account expiry is unknown yet.
    public var accountExpiry: Date?

    public init(
        identifier: UUID = UUID(),
        isDeviceRevoked: Bool? = nil,
        accountExpiry: Date? = nil
    ) {
        self.identifier = identifier
        self.isDeviceRevoked = isDeviceRevoked
        self.accountExpiry = accountExpiry
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

    public init(
        lastErrors: [PacketTunnelErrorWrapper] = [],
        isNetworkReachable: Bool = true,
        deviceCheck: DeviceCheck? = nil,
        tunnelRelay: PacketTunnelRelay? = nil
    ) {
        self.lastErrors = lastErrors
        self.isNetworkReachable = isNetworkReachable
        self.deviceCheck = deviceCheck
        self.tunnelRelay = tunnelRelay
    }
}
