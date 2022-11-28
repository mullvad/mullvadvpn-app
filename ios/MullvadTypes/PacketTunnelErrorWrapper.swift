//
//  PacketTunnelErrorWrapper.swift
//  MullvadTypes
//
//  Created by Sajad Vishkai on 2022-11-28.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

public enum PacketTunnelErrorWrapper: Codable, Equatable, LocalizedError {
    /// Failure that indicates wire guard errors.
    case wireguard(error: String)

    /// Failure that indicates settings need migration.
    case settingsMigration

    public var errorDescription: String? {
        switch self {
        case .wireguard(let error):
            return error
        case .settingsMigration:
            return "Failure to read settings."
        }
    }
}
