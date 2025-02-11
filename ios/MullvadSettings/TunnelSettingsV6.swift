//
//  TunnelSettingsV6.swift
//  MullvadSettings
//
//  Created by Mojgan on 2024-08-08.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

public struct TunnelSettingsV6: Codable, Equatable, TunnelSettings, Sendable {
    /// Relay constraints.
    public var relayConstraints: RelayConstraints

    /// DNS settings.
    public var dnsSettings: DNSSettings

    /// WireGuard obfuscation settings
    public var wireGuardObfuscation: WireGuardObfuscationSettings

    /// Whether Post Quantum exchanges are enabled.
    public var tunnelQuantumResistance: TunnelQuantumResistance

    /// Whether Multihop is enabled.
    public var tunnelMultihopState: MultihopState

    /// DAITA settings.
    public var daita: DAITASettings

    public init(
        relayConstraints: RelayConstraints = RelayConstraints(),
        dnsSettings: DNSSettings = DNSSettings(),
        wireGuardObfuscation: WireGuardObfuscationSettings = WireGuardObfuscationSettings(),
        tunnelQuantumResistance: TunnelQuantumResistance = .automatic,
        tunnelMultihopState: MultihopState = .off,
        daita: DAITASettings = DAITASettings()
    ) {
        self.relayConstraints = relayConstraints
        self.dnsSettings = dnsSettings
        self.wireGuardObfuscation = wireGuardObfuscation
        self.tunnelQuantumResistance = tunnelQuantumResistance
        self.tunnelMultihopState = tunnelMultihopState
        self.daita = daita
    }

    public func upgradeToNextVersion() -> any TunnelSettings {
        TunnelSettingsV7(
            relayConstraints: relayConstraints,
            dnsSettings: dnsSettings,
            wireGuardObfuscation: wireGuardObfuscation,
            tunnelQuantumResistance: tunnelQuantumResistance,
            tunnelMultihopState: tunnelMultihopState,
            daita: daita,
            localNetworkSharing: false,
            includeAllNetworks: false
        )
    }
}
