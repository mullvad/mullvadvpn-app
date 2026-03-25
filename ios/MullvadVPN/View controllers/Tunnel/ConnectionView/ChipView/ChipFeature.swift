//
//  ChipFeature.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-12-06.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//
import MullvadSettings
import MullvadTypes
import PacketTunnelCore
import SwiftUI

protocol ChipFeature: Identifiable {
    var id: FeatureType { get }
    var isEnabled: Bool { get }
    var name: String { get }
    var icon: Image? { get }
}

extension ChipFeature {
    var icon: Image? { nil }
}

enum FeatureType {
    case daita
    case multihop
    case quantumResistance
    case obfuscation
    case dns
    case ipOverrides
    case includeAllNetworks
    case localNetworkSharing
    case ipVersion
}

struct DaitaFeature: ChipFeature {
    let id: FeatureType = .daita
    let state: TunnelState
    let settings: LatestTunnelSettings

    var isEnabled: Bool {
        state.isDaita ?? false
    }

    var name: String {
        NSLocalizedString("DAITA", comment: "")
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
        state.isMultihop
    }

    var name: String {
        NSLocalizedString("Multihop", comment: "")
    }

    var icon: Image? {
        settings.tunnelMultihopState.isWhenNeeded ? .mullvadIconMultihopWhenNeeded : nil
    }
}

struct ObfuscationFeature: ChipFeature {
    let id: FeatureType = .obfuscation
    let settings: LatestTunnelSettings
    let state: ObservedState

    var actualObfuscationMethod: ObfuscationMethod {
        state.connectionState.map { $0.obfuscationMethod } ?? .off
    }

    var isEnabled: Bool {
        actualObfuscationMethod.isEnabled
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

    var isEnabled: Bool {
        guard let selectedRelays = state.relays else {
            return false
        }
        return (selectedRelays.entry?.isIPOverridden ?? false) || selectedRelays.exit.isIPOverridden
    }

    var name: String {
        NSLocalizedString("Server IP override", comment: "")
    }
}

struct IncludeAllNetworksFeature: ChipFeature {
    let id: FeatureType = .includeAllNetworks
    let settings: LatestTunnelSettings

    var isEnabled: Bool {
        let settings = IncludeAllNetworksSettings(
            includeAllNetworksState: settings.includeAllNetworks.includeAllNetworksState,
            localNetworkSharingState: settings.includeAllNetworks.localNetworkSharingState
        )

        return settings.includeAllNetworksIsEnabled
    }

    var name: String {
        NSLocalizedString("Force all apps", comment: "")
    }
}

struct LocalNetworkSharingFeature: ChipFeature {
    let id: FeatureType = .localNetworkSharing
    let settings: LatestTunnelSettings

    var isEnabled: Bool {
        let settings = IncludeAllNetworksSettings(
            includeAllNetworksState: settings.includeAllNetworks.includeAllNetworksState,
            localNetworkSharingState: settings.includeAllNetworks.localNetworkSharingState
        )

        return settings.localNetworkSharingIsEnabled
    }

    var name: String {
        NSLocalizedString("Local network sharing", comment: "")
    }
}

struct IPVersionFeature: ChipFeature {
    let id: FeatureType = .ipVersion
    let state: TunnelState

    var isEnabled: Bool {
        // Show IPv6 indicator when the ingress endpoint is using IPv6
        guard let endpoint = state.relays?.ingress.endpoint else { return false }
        if case .ipv6 = endpoint.socketAddress {
            return true
        }
        return false
    }

    var name: String {
        NSLocalizedString("IPv6", comment: "")
    }
}
