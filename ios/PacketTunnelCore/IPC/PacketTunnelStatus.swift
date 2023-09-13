//
//  PacketTunnelStatus.swift
//  PacketTunnelCore
//
//  Created by pronebird on 27/07/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

public enum BlockStateReason: String, Codable, Equatable {
    /// Device is locked.
    case deviceLocked

    /// Settings schema is outdated.
    case outdatedSchema

    /// No relay satisfying constraints.
    case noRelaysSatisfyingConstraints

    /// Read error.
    case readFailure

    /// Invalid account
    case invalidAccount

    /// Device revoked
    case deviceRevoked
}

/// Struct describing packet tunnel process status.
public struct PacketTunnelStatus: Codable, Equatable {
    /// ???
    public var blockStateReason: BlockStateReason?

    /// Flag indicating whether network is reachable.
    public var isNetworkReachable: Bool

    /// The date of last performed key rotation during device check.
    public var lastKeyRotation: Date?

    /// Current relay.
    public var tunnelRelay: PacketTunnelRelay?

    /// Number of consecutive connection failure attempts.
    public var numberOfFailedAttempts: UInt

    public init(
        blockStateReason: BlockStateReason? = nil,
        isNetworkReachable: Bool = true,
        lastKeyRotation: Date? = nil,
        tunnelRelay: PacketTunnelRelay? = nil,
        numberOfFailedAttempts: UInt = 0
    ) {
        self.blockStateReason = blockStateReason
        self.isNetworkReachable = isNetworkReachable
        self.lastKeyRotation = lastKeyRotation
        self.tunnelRelay = tunnelRelay
        self.numberOfFailedAttempts = numberOfFailedAttempts
    }
}
