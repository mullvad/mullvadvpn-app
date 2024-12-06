//
//  FeatureChipViewModel.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-12-05.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings
import SwiftUICore
class FeatureChipViewModel: ChipViewModelProtocol {
    @Published var chips: [ChipModel] = []

    let tunnelManager: TunnelManager
    var observer: TunnelObserver?

    init(tunnelManager: TunnelManager) {
        self.tunnelManager = tunnelManager
        let observer = TunnelBlockObserver(
            didLoadConfiguration: { [weak self] tunnelManager in
                guard let self else { return }
                chips = createChips(tunnelManager.settings)
            },
            didUpdateTunnelSettings: { [weak self] _, latestTunnelSettings in
                guard let self else { return }
                chips = createChips(latestTunnelSettings)
            }
        )
        self.observer = observer
        tunnelManager.addObserver(observer)
    }

    private func createChips(_ latestTunnelSettings: LatestTunnelSettings) -> [ChipModel] {
        let features: [ChipFeature] = [
            DaitaFeature(settings: latestTunnelSettings),
            QuantumResistanceFeature(settings: latestTunnelSettings),
            MultihopFeature(settings: latestTunnelSettings),
            ObfuscationFeature(settings: latestTunnelSettings),
            DNSFeature(settings: latestTunnelSettings),
        ]

        return features
            .filter { $0.isEnabled }
            .map { ChipModel(name: $0.chipName()) }
    }
}
