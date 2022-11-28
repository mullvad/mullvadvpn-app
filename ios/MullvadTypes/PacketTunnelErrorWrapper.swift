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

    /// Failure to read stored settings.
    case readConfiguration

    public var errorDescription: String? {
        switch self {
        case let .wireguard(error):
            return error
        case .readConfiguration:
            return "Failure to read settings."
        }
    }

    public static func == (lhs: PacketTunnelErrorWrapper, rhs: PacketTunnelErrorWrapper) -> Bool {
        switch (lhs, rhs) {
        case (.readConfiguration, .readConfiguration):
            return true

        case let (.wireguard(error: lhsError), .wireguard(error: rhsError)):
            return lhsError == rhsError

        default:
            return false
        }
    }
}
