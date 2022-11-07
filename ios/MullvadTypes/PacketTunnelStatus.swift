//
//  PacketTunnelStatus.swift
//  MullvadTypes
//
//  Created by pronebird on 27/07/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

public struct DeviceCheck: Codable, Equatable {
    public var lastUpdateDate: Date
    /// Flag indicating device is revoked or not.
    public var isDeviceRevoked: Bool

    /// Flag indicating that account expiry should be set again.
    public var accountExpiry: Date?
}

/// Struct describing packet tunnel process status.
public struct PacketTunnelStatus: Codable, Equatable {
    /// Last tunnel error.
    public var lastError: String?

    /// Flag indicating whether network is reachable.
    public var isNetworkReachable: Bool

    public var deviceCheck: DeviceCheck?
    /// Current relay.
    public var tunnelRelay: PacketTunnelRelay?

    public init(
        lastError: String? = nil,
        isNetworkReachable: Bool = true,
        isDeviceRevoked: Bool = false,
        accountExpiry: Date? = nil,
        tunnelRelay: PacketTunnelRelay? = nil
    ) {
        self.lastError = lastError
        self.isNetworkReachable = isNetworkReachable
        deviceCheck = DeviceCheck(
            lastUpdateDate: Date(),
            isDeviceRevoked: isDeviceRevoked,
            accountExpiry: accountExpiry
        )
        self.tunnelRelay = tunnelRelay
    }

    public init(
        lastError: String? = nil,
        isNetworkReachable: Bool = true,
        deviceCheck: DeviceCheck,
        tunnelRelay: PacketTunnelRelay? = nil
    ) {
        self.lastError = lastError
        self.isNetworkReachable = isNetworkReachable
        self.deviceCheck = deviceCheck
        self.tunnelRelay = tunnelRelay
    }
}
