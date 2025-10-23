//
//  ChipFeature.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-12-06.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//
import MullvadSettings
import PacketTunnelCore
import SwiftUI

protocol ChipFeature: Identifiable {
    var id: FeatureType { get }
    var isEnabled: Bool { get }
    var name: String { get }
}

enum FeatureType {
    case daita
    case multihop
    case quantumResistance
    case obfuscation
    case dns
    case ipOverrides
}

struct DaitaFeature: ChipFeature {
    let id: FeatureType = .daita
    let state: TunnelState
    let settings: LatestTunnelSettings

    var isEnabled: Bool {
        state.isDaita ?? false
    }

    var name: String {
        // When multihop is enabled via DAITA without being explicitly enabled
        // by the user, display combined indicator instead.
        state.isMultihop && !settings.tunnelMultihopState.isEnabled
            ? "\(NSLocalizedString("DAITA", comment: "")): \(NSLocalizedString("Multihop", comment: ""))"
            : NSLocalizedString("DAITA", comment: "")
    }
}

struct QuantumResistanceFeature: ChipFeature {
    let id: FeatureType = .quantumResistance
    let state: TunnelState

    var isEnabled: Bool {
        state.isPostQuantum ?? false
    }

    var name: String {
        NSLocalizedString("Quantum resistance", comment: "")
    }
}

struct MultihopFeature: ChipFeature {
    let id: FeatureType = .multihop
    let state: TunnelState
    let settings: LatestTunnelSettings

    var isEnabled: Bool {
        // Multihop indicator should only be visible when user has explicitly turned on
        // multihop, not when using multihop via DAITA.
        state.isMultihop && settings.tunnelMultihopState.isEnabled
    }

    var name: String {
        NSLocalizedString("Multihop", comment: "")
    }
}

struct ObfuscationFeature: ChipFeature {
    let id: FeatureType = .obfuscation
    let settings: LatestTunnelSettings
    let state: ObservedState

    var actualObfuscationMethod: WireGuardObfuscationState {
        state.connectionState.map { $0.obfuscationMethod } ?? .off
    }

    var isEnabled: Bool {
        actualObfuscationMethod != .off
    }

    var isAutomatic: Bool {
        settings.wireGuardObfuscation.state == .automatic
    }

    var name: String {
        // This just currently says "Obfuscation".
        // To add an automaticity indicator (a trailing " (automatic)"
        // or a colour/border style or whatever), use the `isAutomatic` field.
        // To say what type of obfuscation it is,
        // we can look at `actualObfuscationMethod`
        NSLocalizedString("Obfuscation", comment: "")
    }
}

struct DNSFeature: ChipFeature {
    let id: FeatureType = .dns
    let settings: LatestTunnelSettings

    var isEnabled: Bool {
        settings.dnsSettings.enableCustomDNS || !settings.dnsSettings.blockingOptions.isEmpty
    }

    var name: String {
        if !settings.dnsSettings.blockingOptions.isEmpty {
            NSLocalizedString("DNS content blockers", comment: "")
        } else {
            NSLocalizedString("Custom DNS", comment: "")
        }
    }
}

struct IPOverrideFeature: ChipFeature {
    let id: FeatureType = .ipOverrides
    let state: TunnelState
    let overrides: [IPOverride]

    var isEnabled: Bool {
        guard
            let endpoint = state.relays?.ingress.endpoint
        else { return false }
        return overrides.contains { override in
            (override.ipv4Address.map { $0 == endpoint.ipv4Relay.ip } ?? false)
                || (override.ipv6Address.map { $0 == endpoint.ipv6Relay?.ip } ?? false)
        }
    }

    var name: String {
        NSLocalizedString("Server IP override", comment: "")
    }
}
