//
//  ChipFeature.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-12-06.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//
import MullvadSettings
import SwiftUI

protocol ChipFeature {
    var isEnabled: Bool { get }
    var name: String { get }
}

struct DaitaFeature: ChipFeature {
    let settings: LatestTunnelSettings

    var isEnabled: Bool {
        settings.daita.daitaState.isEnabled
    }

    var name: String {
        String("DAITA")
    }
}

struct QuantumResistanceFeature: ChipFeature {
    let settings: LatestTunnelSettings
    var isEnabled: Bool {
        settings.tunnelQuantumResistance.isEnabled
    }

    var name: String {
        String("Quantum resistance")
    }
}

struct MultihopFeature: ChipFeature {
    let settings: LatestTunnelSettings
    var isEnabled: Bool {
        settings.tunnelMultihopState.isEnabled
    }

    var name: String {
        String("Multihop")
    }
}

struct ObfuscationFeature: ChipFeature {
    let settings: LatestTunnelSettings

    var isEnabled: Bool {
        settings.wireGuardObfuscation.state.isEnabled
    }

    var name: String {
        String("Obfuscation")
    }
}

struct DNSFeature: ChipFeature {
    let settings: LatestTunnelSettings

    var isEnabled: Bool {
        settings.dnsSettings.enableCustomDNS || !settings.dnsSettings.blockingOptions.isEmpty
    }

    var name: String {
        if !settings.dnsSettings.blockingOptions.isEmpty {
            return String("DNS content blockers")
        }
        return String("Custom DNS")
    }
}

struct IPOverrideFeature: ChipFeature {
    let overrides: [IPOverride]

    var isEnabled: Bool {
        !overrides.isEmpty
    }

    var name: String {
        String("Server IP override")
    }
}
