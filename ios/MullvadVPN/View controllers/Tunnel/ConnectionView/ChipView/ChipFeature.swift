// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only
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
    #if NEVER_IN_PRODUCTION
        case gotaTun
    #endif

    /// Accessibility identifier of the chip representing this feature, where one is needed
    /// to tell the chip apart from its label.
    var accessibilityId: AccessibilityIdentifier? {
        switch self {
        case .obfuscation:
            .obfuscationFeatureIndicator
        #if NEVER_IN_PRODUCTION
            case .gotaTun:
                nil
        #endif
        case .daita, .multihop, .quantumResistance, .dns, .ipOverrides, .includeAllNetworks, .localNetworkSharing,
            .ipVersion:
            nil
        }
    }
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

    /// The obfuscation method in use, or `nil` while it is still undetermined.
    ///
    /// The setting names the method up front unless it is `.automatic`, in which case the relay
    /// selector picks one per connection attempt and we have to wait for the tunnel to report it.
    private var method: WireGuardObfuscationState? {
        guard case .automatic = settings.wireGuardObfuscation.state else {
            return settings.wireGuardObfuscation.state
        }
        return state.connectionState.map { connectionState -> WireGuardObfuscationState in
            switch connectionState.obfuscationMethod {
            case .off:
                .off
            case .udpOverTcp:
                .udpOverTcp
            case .shadowsocks:
                .shadowsocks
            case .quic:
                .quic
            case .lwo:
                .lwo
            }
        }
    }

    var isEnabled: Bool {
        method?.isEnabled ?? false
    }

    var name: String {
        switch method {
        case .udpOverTcp, .on:
            NSLocalizedString("UDP-over-TCP", comment: "")
        case .shadowsocks:
            NSLocalizedString("Shadowsocks", comment: "")
        case .quic:
            NSLocalizedString("QUIC", comment: "")
        case .lwo:
            NSLocalizedString("LWO", comment: "")
        case .automatic, .off, nil:
            ""
        }
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

#if NEVER_IN_PRODUCTION
    struct GotaTunFeature: ChipFeature {
        let id: FeatureType = .gotaTun

        var isEnabled: Bool {
            PacketTunnelDebugSettings.useGotaTun
        }

        let name = "GotaTun"
    }
#endif
