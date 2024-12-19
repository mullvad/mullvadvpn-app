//
//  FeatureIndicatorsViewModel.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-12-05.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings

class FeatureIndicatorsViewModel: ChipViewModelProtocol {
    @Published var tunnelSettings: LatestTunnelSettings
    @Published var ipOverrides: [IPOverride]
    @Published var isExpanded = false

    init(tunnelSettings: LatestTunnelSettings, ipOverrides: [IPOverride], isExpanded: Bool = false) {
        self.tunnelSettings = tunnelSettings
        self.ipOverrides = ipOverrides
        self.isExpanded = isExpanded
    }

    var chips: [ChipModel] {
        let features: [ChipFeature] = [
            DaitaFeature(settings: tunnelSettings),
            QuantumResistanceFeature(settings: tunnelSettings),
            MultihopFeature(settings: tunnelSettings),
            ObfuscationFeature(settings: tunnelSettings),
            DNSFeature(settings: tunnelSettings),
            IPOverrideFeature(overrides: ipOverrides),
        ]

        return features
            .filter { $0.isEnabled }
            .map { ChipModel(name: $0.name) }
    }
}
