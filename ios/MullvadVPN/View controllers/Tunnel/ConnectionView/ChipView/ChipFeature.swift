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

// Opting to use NSLocalizedString instead of LocalizedStringKey here in order
// to be able to fetch the string value at a later point (eg. in ChipViewModelProtocol,
// when calculating the text widths of the chips).

protocol ChipFeature {
    var isEnabled: Bool { get }
    var name: String { get }
}

struct DaitaFeature: ChipFeature {
    let state: TunnelState

    var isEnabled: Bool {
        state.isDaita ?? false
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
    let state: TunnelState

    var isEnabled: Bool {
        state.isPostQuantum ?? false
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
    let state: TunnelState
    var isEnabled: Bool {
        state.isMultihop
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
