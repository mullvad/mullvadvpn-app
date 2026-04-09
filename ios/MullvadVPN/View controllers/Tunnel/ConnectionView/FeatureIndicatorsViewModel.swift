//
//  FeatureIndicatorsViewModel.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-12-05.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import PacketTunnelCore
import SwiftUI

class FeatureIndicatorsViewModel: ChipViewModelProtocol {
    @Published var tunnelSettings: LatestTunnelSettings
    @Published var tunnelState: TunnelState
    @Published var observedState: ObservedState
    var onFeaturePressed: ((FeatureType) -> Void)?
    init(
        tunnelSettings: LatestTunnelSettings,
        tunnelStatus: TunnelStatus
    ) {
        self.tunnelSettings = tunnelSettings
        self.tunnelState = tunnelStatus.state
        self.observedState = tunnelStatus.observedState
    }

    var chips: [ChipModel] {
        var features: [any ChipFeature] = []

        // Here can be a check if a feature indicator should show in other connection states
        // e.g. Access local network in blocked state
        switch tunnelState {
        case .connecting, .reconnecting, .negotiatingEphemeralPeer,
            .connected, .pendingReconnect:
            features = [
                DaitaFeature(state: tunnelState, settings: tunnelSettings),
                QuantumResistanceFeature(state: tunnelState),
                MultihopFeature(state: tunnelState, settings: tunnelSettings),
                ObfuscationFeature(settings: tunnelSettings, state: observedState),
                DNSFeature(settings: tunnelSettings),
                IPOverrideFeature(state: tunnelState),
                IncludeAllNetworksFeature(settings: tunnelSettings),
                LocalNetworkSharingFeature(settings: tunnelSettings),
                IPVersionFeature(state: tunnelState),
            ]

        case .error, .waitingForConnectivity:
            features = [
                IncludeAllNetworksFeature(settings: tunnelSettings),
                LocalNetworkSharingFeature(settings: tunnelSettings),
            ]

        default:
            break
        }
        #if NEVER_IN_PRODUCTION
            features.append(GotaTunFeature())
        #endif

        return
            features
            .filter { $0.isEnabled }
            .map { ChipModel(id: $0.id, name: $0.name, icon: $0.icon, style: $0.style) }
    }

    func onPressed(item: ChipModel) {
        onFeaturePressed?(item.id)
    }

    #if NEVER_IN_PRODUCTION
    /// Forces chips to recompute (e.g. after toggling a debug setting).
    func invalidateChips() {
        objectWillChange.send()
    }
    #endif
}
