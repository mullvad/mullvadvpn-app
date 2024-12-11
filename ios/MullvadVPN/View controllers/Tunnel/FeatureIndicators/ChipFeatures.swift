//
//  ChipFeatures.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-12-06.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//
import Foundation
import MullvadSettings
import SwiftUI

protocol ChipFeature {
    var isEnabled: Bool { get }
    var name: LocalizedStringKey { get }
}

struct DaitaFeature: ChipFeature {
    let settings: LatestTunnelSettings

    var isEnabled: Bool {
        settings.daita.daitaState.isEnabled
    }

    var name: LocalizedStringKey {
        LocalizedStringKey("DAITA")
    }
}

struct QuantumResistanceFeature: ChipFeature {
    let settings: LatestTunnelSettings
    var isEnabled: Bool {
        settings.tunnelQuantumResistance.isEnabled
    }

    var name: LocalizedStringKey {
        LocalizedStringKey("Quantum resistance")
    }
}

struct MultihopFeature: ChipFeature {
    let settings: LatestTunnelSettings
    var isEnabled: Bool {
        settings.tunnelMultihopState.isEnabled
    }

    var name: LocalizedStringKey {
        LocalizedStringKey("Multihop")
    }
}

struct ObfuscationFeature: ChipFeature {
    let settings: LatestTunnelSettings

    var isEnabled: Bool {
        settings.wireGuardObfuscation.state.isEnabled
    }

    var name: LocalizedStringKey {
        LocalizedStringKey("Obfuscation")
    }
}

struct DNSFeature: ChipFeature {
    let settings: LatestTunnelSettings

    var isEnabled: Bool {
        settings.dnsSettings.enableCustomDNS || !settings.dnsSettings.blockingOptions.isEmpty
    }

    var name: LocalizedStringKey {
        if !settings.dnsSettings.blockingOptions.isEmpty {
            return LocalizedStringKey("DNS content blockers")
        }
        return LocalizedStringKey("Custom DNS")
    }
}

struct IPOverrideFeature: ChipFeature {
    let overrides: [IPOverride]

    var isEnabled: Bool {
        !overrides.isEmpty
    }

    var name: LocalizedStringKey {
        LocalizedStringKey("Server IP override")
    }
}
