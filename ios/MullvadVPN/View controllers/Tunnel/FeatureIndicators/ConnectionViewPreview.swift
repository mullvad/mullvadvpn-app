//
//  ConnectionViewPreview.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-12-18.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import MullvadMockData
import MullvadSettings
import MullvadTypes
import PacketTunnelCore
import SwiftUI

struct ConnectionViewPreview {
    enum Configuration {
        case normal, normalNoIndicators, expanded, expandedNoIndicators
    }

    private let configuration: Configuration

    private let populatedTunnelSettings = LatestTunnelSettings(
        wireGuardObfuscation: WireGuardObfuscationSettings(state: .udpOverTcp),
        tunnelQuantumResistance: .on,
        tunnelMultihopState: .on,
        daita: DAITASettings(daitaState: .on)
    )

    private let viewModel = ConnectionViewViewModel(
        tunnelStatus: TunnelStatus(
            observedState: .connected(ObservedConnectionState(
                selectedRelays: SelectedRelaysStub.selectedRelays,
                relayConstraints: RelayConstraints(entryLocations: .any, exitLocations: .any, port: .any, filter: .any),
                networkReachability: .reachable,
                connectionAttemptCount: 0,
                transportLayer: .udp,
                remotePort: 80,
                isPostQuantum: true,
                isDaitaEnabled: true
            )),
            state: .connected(SelectedRelaysStub.selectedRelays, isPostQuantum: true, isDaita: true)
        )
    )

    init(configuration: Configuration) {
        self.configuration = configuration
    }

    @ViewBuilder
    func make() -> some View {
        VStack {
            switch configuration {
            case .normal:
                connectionView(with: populatedTunnelSettings, viewModel: viewModel)
            case .normalNoIndicators:
                connectionView(with: LatestTunnelSettings(), viewModel: viewModel)
            case .expanded:
                connectionView(with: populatedTunnelSettings, viewModel: viewModel, isExpanded: true)
            case .expandedNoIndicators:
                connectionView(with: LatestTunnelSettings(), viewModel: viewModel, isExpanded: true)
            }
        }
        .background(UIColor.secondaryColor.color)
    }

    @ViewBuilder
    private func connectionView(
        with settings: LatestTunnelSettings,
        viewModel: ConnectionViewViewModel,
        isExpanded: Bool = false
    ) -> some View {
        ConnectionView(
            connectionViewModel: viewModel,
            indicatorsViewModel: FeatureIndicatorsViewModel(
                tunnelSettings: settings,
                ipOverrides: []
            ),
            isExpanded: isExpanded
        )
    }
}
