//
//  ConnectionViewComponentPreview.swift
//  MullvadVPN
//
//  Created by Andrew Bulhak on 2025-01-03.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadMockData
import MullvadREST
import MullvadSettings
import MullvadTypes
import PacketTunnelCore
import SwiftUI

struct ConnectionViewComponentPreview<Content: View>: View {
    let showIndicators: Bool
    let connectedTunnelStatus = TunnelStatus(
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

    private var tunnelSettings: LatestTunnelSettings {
        LatestTunnelSettings(
            wireGuardObfuscation: WireGuardObfuscationSettings(state: showIndicators ? .udpOverTcp : .off),
            tunnelQuantumResistance: showIndicators ? .on : .off,
            tunnelMultihopState: showIndicators ? .on : .off,
            daita: DAITASettings(daitaState: showIndicators ? .on : .off)
        )
    }

    private let viewModel: ConnectionViewViewModel

    var content: (FeatureIndicatorsViewModel, ConnectionViewViewModel, Binding<Bool>) -> Content

    @State var isExpanded = false

    init(
        showIndicators: Bool,
        content: @escaping (FeatureIndicatorsViewModel, ConnectionViewViewModel, Binding<Bool>) -> Content
    ) {
        self.showIndicators = showIndicators
        self.content = content
        viewModel = ConnectionViewViewModel(
            tunnelStatus: connectedTunnelStatus,
            relayConstraints: RelayConstraints(),
            relayCache: RelayCache(cacheDirectory: ApplicationConfiguration.containerURL),
            customListRepository: CustomListRepository()
        )
        viewModel.outgoingConnectionInfo = OutgoingConnectionInfo(
            ipv4: .init(ip: .allHostsGroup, exitIP: true),
            ipv6: IPV6ConnectionData(
                ip: .broadcast,
                exitIP: true
            )
        )
    }

    var body: some View {
        content(
            FeatureIndicatorsViewModel(
                tunnelSettings: tunnelSettings,
                ipOverrides: [],
                tunnelState: connectedTunnelStatus.state,
                observedState: connectedTunnelStatus.observedState
            ),
            viewModel,
            $isExpanded
        )
        .background(UIColor.secondaryColor.color)
    }
}
