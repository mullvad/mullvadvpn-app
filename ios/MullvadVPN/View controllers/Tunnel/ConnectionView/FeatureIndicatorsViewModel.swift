// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

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
            .map { ChipModel(id: $0.id, name: $0.name, icon: $0.icon) }
    }

    func onPressed(item: ChipModel) {
        onFeaturePressed?(item.id)
    }
}
