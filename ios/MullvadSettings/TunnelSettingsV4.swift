//
//  TunnelSettingsV4.swift
//  MullvadSettings
//
//  Created by Marco Nikic on 2024-02-06.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

public struct TunnelSettingsV4: Codable, Equatable, TunnelSettings {
    /// Relay constraints.
    public var relayConstraints: RelayConstraints

    /// DNS settings.
    public var dnsSettings: DNSSettings

    /// WireGuard obfuscation settings
    public var wireGuardObfuscation: WireGuardObfuscationSettings

    /// Whether Post Quantum exchanges are enabled.
    public var wireGuardTunnelQuantumResistance: WireGuardTunnelQuantumResistanceSettings

    public init(
        relayConstraints: RelayConstraints = RelayConstraints(),
        dnsSettings: DNSSettings = DNSSettings(),
        wireGuardObfuscation: WireGuardObfuscationSettings = WireGuardObfuscationSettings(),
        wireGuardTunnelQuantumResistance: WireGuardTunnelQuantumResistanceSettings = WireGuardTunnelQuantumResistanceSettings()
    ) {
        self.relayConstraints = relayConstraints
        self.dnsSettings = dnsSettings
        self.wireGuardObfuscation = wireGuardObfuscation
        self.wireGuardTunnelQuantumResistance = wireGuardTunnelQuantumResistance
    }

    public func upgradeToNextVersion() -> any TunnelSettings {
        self
    }
}
