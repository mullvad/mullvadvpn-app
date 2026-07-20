//
//  ConnectionViewComponentPreview.swift
//  MullvadVPN
//
//  Created by Andrew Bulhak on 2025-01-03.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadMockData
import MullvadREST
import MullvadSettings
import MullvadTypes
import PacketTunnelCore
import SwiftUI

struct ConnectionViewComponentPreview<Content: View>: View {
    let settingsManager = SettingsManager()
    let showIndicators: Bool
    let connectedTunnelStatus = TunnelStatus(
        observedState: .connected(
            ObservedConnectionState(
                selectedRelays: RelaySelectorStub.selectedRelays,
                relayConstraints: RelayConstraints(
                    entryLocations: .any,
                    exitLocations: .any,
                    port: .any,
                    entryFilter: .any,
                    exitFilter: .any
                ),
                networkReachability: .reachable,
                connectionAttemptCount: 0,
                transportLayer: .udp,
                remotePort: 80,
                isPostQuantum: true,
                isDaitaEnabled: true
            )),
        state:
            .connected(
                RelaySelectorStub.selectedRelays,
                isPostQuantum: true,
                isDaita: true
            )
    )
    let disconnectedTunnelStatus = TunnelStatus(
        observedState: .disconnected,
        state: .disconnected
    )

    private var tunnelSettings: LatestTunnelSettings {
        LatestTunnelSettings(
            wireGuardObfuscation: WireGuardObfuscationSettings(state: showIndicators ? .udpOverTcp : .off),
            tunnelQuantumResistance: showIndicators ? .on : .off,
            tunnelMultihopState: showIndicators ? .always : .never,
            daita: DAITASettings(daitaState: showIndicators ? .on : .off)
        )
    }

    private let viewModel: ConnectionViewViewModel

    var content: (FeatureIndicatorsViewModel, ConnectionViewViewModel, Binding<Bool>) -> Content

    @State var isExpanded = false

    init(
        showIndicators: Bool,
        isConnected: Bool = true,
        content: @escaping (FeatureIndicatorsViewModel, ConnectionViewViewModel, Binding<Bool>) -> Content
    ) {
        self.showIndicators = showIndicators
        self.content = content
        viewModel = ConnectionViewViewModel(
            tunnelStatus: isConnected ? connectedTunnelStatus : disconnectedTunnelStatus,
            relayConstraints: RelayConstraints(),
            relayCacheTracker: MockRelayCacheTracker(),
            customListRepository: CustomListRepository(settingsStore: settingsManager.store)
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
                tunnelStatus: connectedTunnelStatus
            ),
            viewModel,
            $isExpanded
        )
        .background(UIColor.secondaryColor.color)
    }
}
