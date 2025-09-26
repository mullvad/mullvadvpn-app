//
//  FeatureIndicatorsViewModel.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-12-05.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import PacketTunnelCore
import SwiftUI

class FeatureIndicatorsViewModel: ChipViewModelProtocol {
    @Published var tunnelSettings: LatestTunnelSettings
    @Published var ipOverrides: [IPOverride]
    @Published var tunnelState: TunnelState
    @Published var observedState: ObservedState
    var onFeaturePressed: ((FeatureType) -> Void)?
    init(
        tunnelSettings: LatestTunnelSettings,
        ipOverrides: [IPOverride],
        tunnelStatus: TunnelStatus
    ) {
        self.tunnelSettings = tunnelSettings
        self.ipOverrides = ipOverrides
        self.tunnelState = tunnelStatus.state
        self.observedState = tunnelStatus.observedState
    }

    var chips: [ChipModel] {
        // Here can be a check if a feature indicator should show in other connection states
        // e.g. Access local network in blocked state
        switch tunnelState {
        case .connecting, .reconnecting, .negotiatingEphemeralPeer,
             .connected, .pendingReconnect:
            let features: [any ChipFeature] = [
                DaitaFeature(state: tunnelState, settings: tunnelSettings),
                QuantumResistanceFeature(state: tunnelState),
                MultihopFeature(state: tunnelState, settings: tunnelSettings),
                ObfuscationFeature(settings: tunnelSettings, tunnelState: tunnelState, observedState: observedState),
                DNSFeature(settings: tunnelSettings),
                IPOverrideFeature(state: tunnelState, overrides: ipOverrides),
            ]

            return features
                .filter { $0.isEnabled }
                .map {
                    if let multihop = $0 as? MultihopFeature {
                        ChipModel(id: multihop.id, name: multihop.name, isMultihopEverywhere: multihop.isMultihopEverywhere)
                    } else {
                        ChipModel(id: $0.id, name: $0.name, isMultihopEverywhere: false)
                    }
                }
        default:
            return []
        }
    }

    func onPressed(item: ChipModel) {
        onFeaturePressed?(item.id)
    }
}
