//
//  TunnelSettingsUpdate.swift
//  MullvadSettings
//
//  Created by Andrew Bulhak on 2024-02-13.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

public enum TunnelSettingsUpdate: Sendable {
    case localNetworkSharing(Bool)
    case includeAllNetworks(Bool)
    case dnsSettings(DNSSettings)
    case obfuscation(WireGuardObfuscationSettings)
    case relayConstraints(RelayConstraints)
    case quantumResistance(TunnelQuantumResistance)
    case multihop(MultihopState)
    case multihopEverywhere(Bool)
    case daita(DAITASettings)
}

extension TunnelSettingsUpdate {
    public func apply(to settings: inout LatestTunnelSettings) {
        switch self {
        case let .localNetworkSharing(enabled):
            settings.localNetworkSharing = enabled
        case let .includeAllNetworks(enabled):
            settings.includeAllNetworks = enabled
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
        case let .multihopEverywhere(enabled):
            settings.multihopEverwhere = enabled
        }
    }

    public var subjectName: String {
        switch self {
        case .localNetworkSharing: "Local network sharing"
        case .includeAllNetworks: "Include all networks"
        case .dnsSettings: "DNS settings"
        case .obfuscation: "obfuscation settings"
        case .relayConstraints: "relay constraints"
        case .quantumResistance: "quantum resistance"
        case .multihop: "multihop"
        case .multihopEverywhere: "multihop everywhere"
        case .daita: "daita"
        }
    }
}
