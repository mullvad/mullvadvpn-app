//
//  TunnelManagerError.swift
//  TunnelManagerError
//
//  Created by pronebird on 07/09/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension TunnelManager {
    /// An error emitted by all public methods of TunnelManager
    enum Error: ChainedError {
        /// Account is unset.
        case unsetAccount

        /// Tunnel is not set yet.
        case unsetTunnel

        /// Failure to start the VPN tunnel via system call.
        case startVPNTunnel(Swift.Error)

        /// Failure to load the system VPN configurations created by the app.
        case loadAllVPNConfigurations(Swift.Error)

        /// Failure to save the system VPN configuration.
        case saveVPNConfiguration(Swift.Error)

        /// Failure to reload the system VPN configuration.
        case reloadVPNConfiguration(Swift.Error)

        /// Failure to remove the system VPN configuration.
        case removeVPNConfiguration(Swift.Error)

        /// Failure to read settings.
        case readSettings(Swift.Error)

        /// Failure to write settings.
        case writeSettings(Swift.Error)

        /// Failure to delete settings.
        case deleteSettings(Swift.Error)

        /// Failure to read relays cache.
        case readRelays

        /// Failure to find a relay satisfying the given constraints.
        case cannotSatisfyRelayConstraints

        /// Failure to create device.
        case createDevice(REST.Error)

        /// Failure to delete device.
        case deleteDevice(REST.Error)

        /// Failure to obtain device data.
        case getDevice(REST.Error)

        /// Requested device is already revoked.
        case deviceRevoked

        /// Failure to obtain account data.
        case getAccountData(REST.Error)

        /// Failure to create account.
        case createAccount(REST.Error)

        /// Failure to rotate WireGuard key.
        case rotateKey(REST.Error)

        /// Failure to reload tunnel.
        case reloadTunnel(Swift.Error)

        var errorDescription: String? {
            switch self {
            case .unsetAccount:
                return "Account is unset."
            case .unsetTunnel:
                return "Tunnel is unset."
            case .startVPNTunnel:
                return "Failed to start the VPN tunnel."
            case .loadAllVPNConfigurations:
                return "Failed to load the system VPN configurations."
            case .saveVPNConfiguration:
                return "Failed to save the system VPN configuration."
            case .reloadVPNConfiguration:
                return "Failed to reload the system VPN configuration."
            case .removeVPNConfiguration:
                return "Failed to remove the system VPN configuration."
            case .readSettings:
                return "Failed to read settings."
            case .readRelays:
                return "Failed to read relays."
            case .cannotSatisfyRelayConstraints:
                return "Failed to satisfy the relay constraints."
            case .writeSettings:
                return "Failed to write settings."
            case .deleteSettings:
                return "Failed to delete settings."
            case .createDevice:
                return "Failed to create a device."
            case .deleteDevice:
                return "Failed to delete a device."
            case .getDevice:
                return "Failed to obtain device data."
            case .deviceRevoked:
                return "Requested device is already revoked."
            case .getAccountData:
                return "Failed to obtain account data."
            case .createAccount:
                return "Failed to create new account."
            case .rotateKey:
                return "Failed to rotate WireGuard key."
            case .reloadTunnel:
                return "Failed to reload tunnel."
            }
        }
    }
}
