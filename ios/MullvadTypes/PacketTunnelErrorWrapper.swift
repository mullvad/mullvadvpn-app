//
//  PacketTunnelErrorWrapper.swift
//  MullvadTypes
//
//  Created by Sajad Vishkai on 2022-11-28.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

public enum PacketTunnelErrorWrapper: Codable, Equatable, LocalizedError {
    public enum ConfigurationFailureCause: Codable, Equatable {
        /// Device is locked.
        case deviceLocked

        /// Settings schema is outdated.
        case outdatedSchema

        /// No relay satisfying constraints.
        case noRelaysSatisfyingConstraints

        /// Read error.
        case readFailure
    }

    /// Failure that indicates WireGuard errors.
    case wireguard(String)

    /// Failure to read stored settings.
    case configuration(ConfigurationFailureCause)

    public var errorDescription: String? {
        switch self {
        case let .wireguard(error):
            return error
        case let .configuration(cause):
            switch cause {
            case .deviceLocked:
                return "Device is locked."
            case .outdatedSchema:
                return "Settings schema is outdated."
            case .readFailure:
                return "Failure to read VPN configuration."
            case .noRelaysSatisfyingConstraints:
                return "No relays satisfying constraints."
            }
        }
    }
}
