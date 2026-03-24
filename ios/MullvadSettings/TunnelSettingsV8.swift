//
//  TunnelSettingsV8.swift
//  MullvadVPN
//
//  Created by Andrew Bulhak on 2026-03-12.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

public struct TunnelSettingsV8: Codable, Equatable, TunnelSettings, Sendable {
    /// Relay constraints.
    public var relayConstraints: RelayConstraints

    /// DNS settings.
    public var dnsSettings: DNSSettings

    /// WireGuard obfuscation settings
    public var wireGuardObfuscation: WireGuardObfuscationSettings

    /// Whether Post Quantum exchanges are enabled.
    public var tunnelQuantumResistance: TunnelQuantumResistance

    /// Whether Multihop is enabled.
    public var tunnelMultihopState: MultihopStateV2

    /// DAITA settings.
    public var daita: DAITASettings

    /// IAN settings.
    public var includeAllNetworks: IncludeAllNetworksSettings

    /// IP version preference for relay connections.
    public var ipVersion: IPVersion

    public var automaticMultihopIsEnabled: Bool {
        (tunnelMultihopState == .whenNeeded)
            || (tunnelMultihopState == .always && relayConstraints.entryLocations == .any)
    }

    public init(
        relayConstraints: RelayConstraints = RelayConstraints(),
        dnsSettings: DNSSettings = DNSSettings(),
        wireGuardObfuscation: WireGuardObfuscationSettings = WireGuardObfuscationSettings(),
        tunnelQuantumResistance: TunnelQuantumResistance = .on,
        tunnelMultihopState: MultihopStateV2 = .never,
        daita: DAITASettings = DAITASettings(),
        includeAllNetworks: IncludeAllNetworksSettings = IncludeAllNetworksSettings(),
        ipVersion: IPVersion = .automatic
    ) {
        self.relayConstraints = relayConstraints
        self.dnsSettings = dnsSettings
        self.wireGuardObfuscation = wireGuardObfuscation
        self.tunnelQuantumResistance = tunnelQuantumResistance
        self.tunnelMultihopState = tunnelMultihopState
        self.daita = daita
        self.includeAllNetworks = includeAllNetworks
        self.ipVersion = ipVersion
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
            try container.decode(MultihopStateV2.self, forKey: .tunnelMultihopState)
        self.daita =
            try container.decode(DAITASettings.self, forKey: .daita)
        self.includeAllNetworks =
            (try? container.decode(IncludeAllNetworksSettings.self, forKey: .includeAllNetworks))
            ?? IncludeAllNetworksSettings()
        self.ipVersion =
            (try? container.decode(IPVersion.self, forKey: .ipVersion))
            ?? .automatic
    }

    public func upgradeToNextVersion() -> any TunnelSettings {
        self
    }

    public var debugDescription: String {
        "TunnelSettingsV8(relayConstraints: \(self.relayConstraints), dnsSettings: \(self.dnsSettings), wireGuardObfuscation: \(self.wireGuardObfuscation), tunnelQuantumResistance: \(self.tunnelQuantumResistance), tunnelMultihopState: \(self.tunnelMultihopState), daita: \(self.daita), includeAllNetworks: \(self.includeAllNetworks.debugDescription))"
    }
}
