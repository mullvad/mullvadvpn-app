//
//  TunnelSettingsV8.swift
//  MullvadVPN
//
//  Created by Emīls Piņķis on 2025-12-30.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
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
    public var tunnelMultihopState: MultihopState

    /// DAITA settings.
    public var daita: DAITASettings

    /// Local networks sharing.
    public var localNetworkSharing: Bool

    /// Forces the system to route most traffic through the tunnel
    public var includeAllNetworks: Bool

    /// IP version preference for relay connections.
    public var ipVersion: IPVersion

    public init(
        relayConstraints: RelayConstraints = RelayConstraints(),
        dnsSettings: DNSSettings = DNSSettings(),
        wireGuardObfuscation: WireGuardObfuscationSettings = WireGuardObfuscationSettings(),
        tunnelQuantumResistance: TunnelQuantumResistance = .automatic,
        tunnelMultihopState: MultihopState = .off,
        daita: DAITASettings = DAITASettings(),
        localNetworkSharing: Bool = false,
        includeAllNetworks: Bool = false,
        ipVersion: IPVersion = .automatic
    ) {
        self.relayConstraints = relayConstraints
        self.dnsSettings = dnsSettings
        self.wireGuardObfuscation = wireGuardObfuscation
        self.tunnelQuantumResistance = tunnelQuantumResistance
        self.tunnelMultihopState = tunnelMultihopState
        self.daita = daita
        self.localNetworkSharing = localNetworkSharing
        self.includeAllNetworks = includeAllNetworks
        self.ipVersion = ipVersion
    }

    public func upgradeToNextVersion() -> any TunnelSettings {
        self
    }
}
