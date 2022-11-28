//
//  PacketTunnelErrorWrapper.swift
//  MullvadTypes
//
//  Created by Sajad Vishkai on 2022-11-28.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

public enum PacketTunnelErrorWrapper: LocalizedError {
    case wireguard(error: String)
    case settingsMigration

    public var errorDescription: String? {
        switch self {
        case .wireguard(let error):
            return error
        case .settingsMigration:
            return "Failure due to read settings."
        }
    }
}
