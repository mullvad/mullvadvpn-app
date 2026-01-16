//
//  TunnelSettingsV6 2.swift
//  MullvadVPN
//
//  Created by Steffen Ernst on 2025-02-04.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
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

    /// Forces the system to route most traffic through the tunnel.
    public var includeAllNetworks: Bool

    /// Consent to enable `includeAllNetworks`, understanding the pros and cons.
    public var includeAllNetworksConsent: Bool

    /// Local network sharing.
    public var localNetworkSharing: Bool

    public init(
        relayConstraints: RelayConstraints = RelayConstraints(),
        dnsSettings: DNSSettings = DNSSettings(),
        wireGuardObfuscation: WireGuardObfuscationSettings = WireGuardObfuscationSettings(),
        tunnelQuantumResistance: TunnelQuantumResistance = .automatic,
        tunnelMultihopState: MultihopState = .off,
        daita: DAITASettings = DAITASettings(),
        includeAllNetworks: InclueAllNetworksSettings = InclueAllNetworksSettings()
    ) {
        self.relayConstraints = relayConstraints
        self.dnsSettings = dnsSettings
        self.wireGuardObfuscation = wireGuardObfuscation
        self.tunnelQuantumResistance = tunnelQuantumResistance
        self.tunnelMultihopState = tunnelMultihopState
        self.daita = daita
        self.includeAllNetworks = includeAllNetworks.localNetworkSharingState.isEnabled
        self.localNetworkSharing = includeAllNetworks.includeAllNetworksState.isEnabled
        self.includeAllNetworksConsent = includeAllNetworks.consent
    }

    public func upgradeToNextVersion() -> any TunnelSettings {
        self
    }
}
