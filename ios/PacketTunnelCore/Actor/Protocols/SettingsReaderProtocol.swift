//
//  SettingsReaderProtocol.swift
//  PacketTunnel
//
//  Created by pronebird on 25/08/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings
import MullvadTypes
import Network
import WireGuardKitTypes

/// A type that implements a reader that can return settings required by `PacketTunnelActor` in order to configure the tunnel.
public protocol SettingsReaderProtocol {
    /**
     Read settings from storage.

     - Throws: an error thrown by this method is passed down to the implementation of `BlockedStateErrorMapperProtocol`.
     - Returns: `Settings` used to configure packet tunnel adapter.
     */
    func read() throws -> Settings
}

/// Struct holding settings necessary to configure packet tunnel adapter.
public struct Settings: Equatable {
    /// Private key used by device.
    public var privateKey: PrivateKey

    /// IP addresses assigned for tunnel interface.
    public var interfaceAddresses: [IPAddressRange]

    /// Relay constraints.
    public var relayConstraints: RelayConstraints

    /// DNS servers selected by user.
    public var dnsServers: SelectedDNSServers

    /// Obfuscation settings
    public var obfuscation: WireGuardObfuscationSettings

    public var quantumResistance: TunnelQuantumResistance

    /// Whether multi-hop is enabled.
    public var multihopState: MultihopState

    public init(
        privateKey: PrivateKey,
        interfaceAddresses: [IPAddressRange],
        relayConstraints: RelayConstraints,
        dnsServers: SelectedDNSServers,
        obfuscation: WireGuardObfuscationSettings,
        quantumResistance: TunnelQuantumResistance,
        multihopState: MultihopState
    ) {
        self.privateKey = privateKey
        self.interfaceAddresses = interfaceAddresses
        self.relayConstraints = relayConstraints
        self.dnsServers = dnsServers
        self.obfuscation = obfuscation
        self.quantumResistance = quantumResistance
        self.multihopState = multihopState
    }
}

/// Enum describing selected DNS servers option.
public enum SelectedDNSServers: Equatable {
    /// Custom DNS servers.
    case custom([IPAddress])
    /// Mullvad server acting as a blocking DNS proxy.
    case blocking(IPAddress)
    /// Gateway IP will be used as DNS automatically.
    case gateway

    public static func == (lhs: SelectedDNSServers, rhs: SelectedDNSServers) -> Bool {
        return switch (lhs, rhs) {
        case let (.custom(lhsAddresss), .custom(rhsAddresses)):
            lhsAddresss.map { $0.rawValue } == rhsAddresses.map { $0.rawValue }
        case let (.blocking(lhsAddress), .blocking(rhsAddress)):
            lhsAddress.rawValue == rhsAddress.rawValue
        case (.gateway, .gateway):
            true
        default: false
        }
    }
}
