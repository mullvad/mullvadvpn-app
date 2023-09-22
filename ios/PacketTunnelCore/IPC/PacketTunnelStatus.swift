//
//  PacketTunnelStatus.swift
//  PacketTunnelCore
//
//  Created by pronebird on 27/07/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

/// Struct describing packet tunnel process status.
public struct PacketTunnelStatus: Codable, Equatable {
    /// The reason why packet tunnel entered error state.
    /// Set to `nil` when tunnel is not in error state.
    public var blockedStateReason: BlockedStateReason?

    /// Flag indicating whether network is reachable.
    public var isNetworkReachable: Bool

    /// The date of last performed key rotation during device check.
    public var lastKeyRotation: Date?

    /// Current relay.
    public var tunnelRelay: PacketTunnelRelay?

    /// Number of consecutive connection failure attempts.
    public var numberOfFailedAttempts: UInt

    public init(
        blockStateReason: BlockedStateReason? = nil,
        isNetworkReachable: Bool = true,
        lastKeyRotation: Date? = nil,
        tunnelRelay: PacketTunnelRelay? = nil,
        numberOfFailedAttempts: UInt = 0
    ) {
        self.blockedStateReason = blockStateReason
        self.isNetworkReachable = isNetworkReachable
        self.lastKeyRotation = lastKeyRotation
        self.tunnelRelay = tunnelRelay
        self.numberOfFailedAttempts = numberOfFailedAttempts
    }
}
