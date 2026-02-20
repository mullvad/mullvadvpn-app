//
//  TunnelSettingsUpdate.swift
//  MullvadSettings
//
//  Created by Andrew Bulhak on 2024-02-13.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

// Note:
// Existing keys in `TunnelSettingsUpdate` must not be removed.
// They are required for backward compatibility.
// If a key is no longer used, mark it as deprecated instead of deleting it.
// Version upgrades should be handled in `upgradeToNextVersion()`.
public enum TunnelSettingsUpdate: Sendable {
    case dnsSettings(DNSSettings)
    case obfuscation(WireGuardObfuscationSettings)
    case relayConstraints(RelayConstraints)
    case quantumResistance(TunnelQuantumResistance)
    case multihop(MultihopState)
    case daita(DAITASettings)
    case includeAllNetworks(IncludeAllNetworksSettings)
}

extension TunnelSettingsUpdate {
    public func apply(to settings: inout LatestTunnelSettings) {
        switch self {
        case let .dnsSettings(newDNSSettings):
            settings.dnsSettings = newDNSSettings
        case let .obfuscation(newObfuscationSettings):
            settings.wireGuardObfuscation = newObfuscationSettings
        case let .relayConstraints(newRelayConstraints):
            settings.relayConstraints = newRelayConstraints
        case let .quantumResistance(newQuantumResistance):
            settings.tunnelQuantumResistance = newQuantumResistance
        case let .multihop(newState):
            settings.tunnelMultihopState = newState
        case let .daita(newDAITASettings):
            settings.daita = newDAITASettings
        case let .includeAllNetworks(newIncludeAllNetworksSettings):
            settings.includeAllNetworks = newIncludeAllNetworksSettings
        }
    }

    public var subjectName: String {
        switch self {
        case .dnsSettings: "DNS settings"
        case .obfuscation: "obfuscation settings"
        case .relayConstraints: "relay constraints"
        case .quantumResistance: "quantum resistance"
        case .multihop: "multihop"
        case .daita: "daita"
        case .includeAllNetworks: "include all networks"
        }
    }
}
