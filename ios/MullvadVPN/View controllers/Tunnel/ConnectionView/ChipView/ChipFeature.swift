//
//  ChipFeature.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-12-06.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//
import MullvadSettings
import SwiftUI

// Opting to use NSLocalizedString instead of LocalizedStringKey here in order
// to be able to fetch the string value at a later point (eg. in ChipViewModelProtocol,
// when calculating the text widths of the chips).

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
        NSLocalizedString(
            "FEATURE_INDICATORS_CHIP_DAITA",
            tableName: "FeatureIndicatorsChip",
            value: "DAITA",
            comment: ""
        )
    }
}

struct QuantumResistanceFeature: ChipFeature {
    let settings: LatestTunnelSettings
    var isEnabled: Bool {
        settings.tunnelQuantumResistance.isEnabled
    }

    var name: String {
        NSLocalizedString(
            "FEATURE_INDICATORS_CHIP_QUANTUM_RESISTANCE",
            tableName: "FeatureIndicatorsChip",
            value: "Quantum resistance",
            comment: ""
        )
    }
}

struct MultihopFeature: ChipFeature {
    let settings: LatestTunnelSettings
    var isEnabled: Bool {
        settings.tunnelMultihopState.isEnabled
    }

    var name: String {
        NSLocalizedString(
            "FEATURE_INDICATORS_CHIP_MULTIHOP",
            tableName: "FeatureIndicatorsChip",
            value: "Multihop",
            comment: ""
        )
    }
}

struct ObfuscationFeature: ChipFeature {
    let settings: LatestTunnelSettings

    var isEnabled: Bool {
        settings.wireGuardObfuscation.state.isEnabled
    }

    var name: String {
        NSLocalizedString(
            "FEATURE_INDICATORS_CHIP_OBFUSCATION",
            tableName: "FeatureIndicatorsChip",
            value: "Obfuscation",
            comment: ""
        )
    }
}

struct DNSFeature: ChipFeature {
    let settings: LatestTunnelSettings

    var isEnabled: Bool {
        settings.dnsSettings.enableCustomDNS || !settings.dnsSettings.blockingOptions.isEmpty
    }

    var name: String {
        if !settings.dnsSettings.blockingOptions.isEmpty {
            NSLocalizedString(
                "FEATURE_INDICATORS_CHIP_CONTENT_BLOCKERS",
                tableName: "FeatureIndicatorsChip",
                value: "DNS content blockers",
                comment: ""
            )
        } else {
            NSLocalizedString(
                "FEATURE_INDICATORS_CHIP_CUSTOM_DNS",
                tableName: "FeatureIndicatorsChip",
                value: "Custom DNS",
                comment: ""
            )
        }
    }
}

struct IPOverrideFeature: ChipFeature {
    let overrides: [IPOverride]

    var isEnabled: Bool {
        !overrides.isEmpty
    }

    var name: String {
        NSLocalizedString(
            "FEATURE_INDICATORS_CHIP_IP_OVERRIDE",
            tableName: "FeatureIndicatorsChip",
            value: "Server IP Override",
            comment: ""
        )
    }
}
