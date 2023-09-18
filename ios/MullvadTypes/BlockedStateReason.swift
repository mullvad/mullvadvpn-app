//
//  BlockedStateReason.swift
//  MullvadTypes
//
//  Created by pronebird on 18/09/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Reason why packet tunnel entered error state.
public enum BlockedStateReason: String, Codable, Equatable {
    /// Device is locked.
    case deviceLocked

    /// Settings schema is outdated.
    case outdatedSchema

    /// No relay satisfying constraints.
    case noRelaysSatisfyingConstraints

    /// Any other failure when reading settings.
    case readSettings

    /// Invalid account.
    case invalidAccount

    /// Device revoked.
    case deviceRevoked

    /// Device is logged out.
    /// This is an extreme edge case, most likely means that main bundle forgot to delete the VPN configuration during logout.
    case deviceLoggedOut

    /// Tunnel adapter error.
    case tunnelAdapter

    /// Unidentified reason.
    case unknown
}
