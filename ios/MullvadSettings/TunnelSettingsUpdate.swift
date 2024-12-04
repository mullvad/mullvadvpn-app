//
//  TunnelSettingsUpdate.swift
//  MullvadSettings
//
//  Created by Andrew Bulhak on 2024-02-13.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

public enum TunnelSettingsUpdate: Sendable {
    case dnsSettings(DNSSettings)
    case obfuscation(WireGuardObfuscationSettings)
    case relayConstraints(RelayConstraints)
    case quantumResistance(TunnelQuantumResistance)
    case multihop(MultihopState)
    case daita(DAITASettings)
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
        }
    }
}
