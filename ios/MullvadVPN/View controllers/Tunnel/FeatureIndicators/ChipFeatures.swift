//
//  ChipFeatures.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-12-06.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//
import Foundation
import MullvadSettings
import SwiftUICore

protocol ChipFeature {
    var isEnabled: Bool { get }
    func name() -> LocalizedStringKey
}

struct DaitaFeature: ChipFeature {
    let settings: LatestTunnelSettings

    var isEnabled: Bool {
        settings.daita.daitaState.isEnabled
    }

    func name() -> LocalizedStringKey {
        LocalizedStringKey("DAITA")
    }
}

struct QuantumResistanceFeature: ChipFeature {
    let settings: LatestTunnelSettings
    var isEnabled: Bool {
        settings.tunnelQuantumResistance.isEnabled
    }

    func name() -> LocalizedStringKey {
        LocalizedStringKey("Quantum resistance")
    }
}

struct MultihopFeature: ChipFeature {
    let settings: LatestTunnelSettings
    var isEnabled: Bool {
        settings.tunnelMultihopState.isEnabled
    }

    func name() -> LocalizedStringKey {
        LocalizedStringKey("Multihop")
    }
}

struct ObfuscationFeature: ChipFeature {
    let settings: LatestTunnelSettings

    var isEnabled: Bool {
        settings.wireGuardObfuscation.state.isEnabled
    }

    func name() -> LocalizedStringKey {
        LocalizedStringKey("Obfuscation")
    }
}

struct DNSFeature: ChipFeature {
    let settings: LatestTunnelSettings

    var isEnabled: Bool {
        settings.dnsSettings.enableCustomDNS || !settings.dnsSettings.blockingOptions.isEmpty
    }

    func name() -> LocalizedStringKey {
        if !settings.dnsSettings.blockingOptions.isEmpty {
            return LocalizedStringKey("DNS content blockers")
        }
        return LocalizedStringKey("Custom DNS")
    }
}

struct IPOverrideFeature: ChipFeature {
    let repository: IPOverrideRepositoryProtocol

    var isEnabled: Bool {
        !repository.fetchAll().isEmpty
    }

    func name() -> LocalizedStringKey {
        LocalizedStringKey("Server IP override")
    }
}
