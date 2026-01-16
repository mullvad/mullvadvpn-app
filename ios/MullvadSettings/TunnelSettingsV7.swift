//
//  TunnelSettingsV6 2.swift
//  MullvadVPN
//
//  Created by Steffen Ernst on 2025-02-04.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

public struct TunnelSettingsV7: Codable, Equatable, TunnelSettings, Sendable {
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

    /// IAN settings.
    public var includeAllNetworks: IncludeAllNetworksSettings

    public init(
        relayConstraints: RelayConstraints = RelayConstraints(),
        dnsSettings: DNSSettings = DNSSettings(),
        wireGuardObfuscation: WireGuardObfuscationSettings = WireGuardObfuscationSettings(),
        tunnelQuantumResistance: TunnelQuantumResistance = .automatic,
        tunnelMultihopState: MultihopState = .off,
        daita: DAITASettings = DAITASettings(),
        includeAllNetworks: IncludeAllNetworksSettings = IncludeAllNetworksSettings()
    ) {
        self.relayConstraints = relayConstraints
        self.dnsSettings = dnsSettings
        self.wireGuardObfuscation = wireGuardObfuscation
        self.tunnelQuantumResistance = tunnelQuantumResistance
        self.tunnelMultihopState = tunnelMultihopState
        self.daita = daita
        self.includeAllNetworks = includeAllNetworks
    }

    public init(from decoder: any Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)

        self.relayConstraints =
            try container.decode(RelayConstraints.self, forKey: .relayConstraints)
        self.dnsSettings =
            try container.decode(DNSSettings.self, forKey: .dnsSettings)
        self.wireGuardObfuscation =
            try container.decode(WireGuardObfuscationSettings.self, forKey: .wireGuardObfuscation)
        self.tunnelQuantumResistance =
            try container.decode(TunnelQuantumResistance.self, forKey: .tunnelQuantumResistance)
        self.tunnelMultihopState =
            try container.decode(MultihopState.self, forKey: .tunnelMultihopState)
        self.daita =
            try container.decode(DAITASettings.self, forKey: .daita)
        self.includeAllNetworks =
            try container.decodeIfPresent(
                IncludeAllNetworksSettings.self,
                forKey: .includeAllNetworks
            ) ?? IncludeAllNetworksSettings()
    }

    public func upgradeToNextVersion() -> any TunnelSettings {
        self
    }
}
