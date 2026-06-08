//
//  MigratedVersion.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-05-27.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

public enum MigratedVersion: Int, Sendable, Equatable {
    case v1 = 1
    case v2 = 2

    var nextVersion: MigratedVersion {
        switch self {
        case .v1:
            .v2
        case .v2:
            .v2
        }
    }

    public static let current: MigratedVersion = .v2

    public static func == (lhs: Self, rhs: Self) -> Bool {
        lhs.rawValue == rhs.rawValue
    }
}
