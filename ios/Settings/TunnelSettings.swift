//
//  TunnelSettings.swift
//  MullvadVPN
//
//  Created by Marco Nikic on 2023-07-31.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Alias to the latest version of the `TunnelSettings`.
public typealias LatestTunnelSettings = TunnelSettingsV2

/// Settings and device state schema versions.
public enum SchemaVersion: Int, Equatable {
    /// Legacy settings format, stored as `TunnelSettingsV1`.
    case v1 = 1

    /// New settings format, stored as `TunnelSettingsV2`.
    case v2 = 2

    /// Current schema version.
    public static let current = SchemaVersion.v2
}
