//
//  SettingsReaderProtocol.swift
//  PacketTunnel
//
//  Created by pronebird on 25/08/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes
import Network
import WireGuardKitTypes

public protocol SettingsReaderProtocol {
    /**
     Read settings from storage.

     - Throws: an error of type `ReadDeviceDataError` when device state is either revoked or logged out. In other cases it's expected to pass internal
     errors.
     - Returns: `Settings` used to configure packet tunnel adapter.
     */
    func read() throws -> Settings
}

/// Struct holding settings necessary to configure packet tunnel adapter.
public struct Settings {
    /// Private key used by device.
    public var privateKey: PrivateKey

    /// IP addresses assigned for tunnel interface.
    public var interfaceAddresses: [IPAddressRange]

    /// Relay constraints.
    public var relayConstraints: RelayConstraints

    /// DNS servers selected by user.
    public var dnsServers: SelectedDNSServers

    public init(
        privateKey: PrivateKey,
        interfaceAddresses: [IPAddressRange],
        relayConstraints: RelayConstraints,
        dnsServers: SelectedDNSServers
    ) {
        self.privateKey = privateKey
        self.interfaceAddresses = interfaceAddresses
        self.relayConstraints = relayConstraints
        self.dnsServers = dnsServers
    }
}

/// Error that implementations of `SettingsReaderProtocol` are expected to throw when device state is either revoked or loggedOut.
public enum ReadDeviceDataError: LocalizedError {
    case loggedOut, revoked

    public var errorDescription: String? {
        switch self {
        case .loggedOut:
            return "Device is logged out."
        case .revoked:
            return "Device is revoked."
        }
    }
}

/// Enum describing selected DNS servers option.
public enum SelectedDNSServers {
    /// Custom DNS servers.
    case custom([IPAddress])
    /// Mullvad server acting as a blocking DNS proxy.
    case blocking(IPAddress)
    /// Gateway IP will be used as DNS automatically.
    case gateway
}
