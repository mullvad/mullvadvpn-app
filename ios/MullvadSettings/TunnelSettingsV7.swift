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

    /// Legacy coding key for the `localNetworkSharing` Bool field that existed prior to its
    /// consolidation into `IncludeAllNetworksSettings`.
    private enum LegacyCodingKeys: String, CodingKey {
        case localNetworkSharing
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

        // Handle both the new struct format and the legacy Bool format for `includeAllNetworks`.
        // Prior to the IAN activation flow, `includeAllNetworks` was a Bool and `localNetworkSharing`
        // was a separate Bool field.
        if let settings = try? container.decode(IncludeAllNetworksSettings.self, forKey: .includeAllNetworks) {
            self.includeAllNetworks = settings
        } else {
            let includeAllNetworks = try container.decodeIfPresent(Bool.self, forKey: .includeAllNetworks) ?? false
            let legacyContainer = try decoder.container(keyedBy: LegacyCodingKeys.self)
            let localNetworkSharing =
                try legacyContainer.decodeIfPresent(
                    Bool.self,
                    forKey: .localNetworkSharing
                ) ?? false
            self.includeAllNetworks = IncludeAllNetworksSettings(
                includeAllNetworksState: includeAllNetworks ? .on : .off,
                localNetworkSharingState: localNetworkSharing ? .on : .off
            )
        }
    }

    public func upgradeToNextVersion() -> any TunnelSettings {
        self
    }
}
