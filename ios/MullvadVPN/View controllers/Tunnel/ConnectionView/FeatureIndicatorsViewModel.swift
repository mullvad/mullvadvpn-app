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
            let features: [ChipFeature] = [
                DaitaFeature(state: tunnelState),
                QuantumResistanceFeature(state: tunnelState),
                MultihopFeature(state: tunnelState),
                ObfuscationFeature(settings: tunnelSettings, state: observedState),
                DNSFeature(settings: tunnelSettings),
                IPOverrideFeature(overrides: ipOverrides),
            ]

            return features
                .filter { $0.isEnabled }
                .map { ChipModel(name: $0.name) }
        default:
            return []
        }
    }
}
