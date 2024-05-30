//
//  TunnelSettings.swift
//  MullvadVPN
//
//  Created by Marco Nikic on 2023-07-31.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Alias to the latest version of the `TunnelSettings`.
public typealias LatestTunnelSettings = TunnelSettingsV5

/// Protocol all TunnelSettings must adhere to, for upgrade purposes.
public protocol TunnelSettings: Codable {
    func upgradeToNextVersion() -> any TunnelSettings
}

/// Settings and device state schema versions.
public enum SchemaVersion: Int, Equatable {
    /// Legacy settings format, stored as `TunnelSettingsV1`.
    case v1 = 1

    /// New settings format, stored as `TunnelSettingsV2`.
    case v2 = 2

    /// V2 format with WireGuard obfuscation options, stored as `TunnelSettingsV3`.
    case v3 = 3

    /// V3 format with post quantum options, stored as `TunnelSettingsV4`.
    case v4 = 4

    /// V4 format with multi-hop options, stored as `TunnelSettingsV5`.
    case v5 = 5

    var settingsType: any TunnelSettings.Type {
        switch self {
        case .v1: return TunnelSettingsV1.self
        case .v2: return TunnelSettingsV2.self
        case .v3: return TunnelSettingsV3.self
        case .v4: return TunnelSettingsV4.self
        case .v5: return TunnelSettingsV5.self
        }
    }

    var nextVersion: Self {
        switch self {
        case .v1: return .v2
        case .v2: return .v3
        case .v3: return .v4
        case .v4: return .v5
        case .v5: return .v5
        }
    }

    /// Current schema version.
    public static let current = SchemaVersion.v5
}
