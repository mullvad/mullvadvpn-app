// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadMockData
import MullvadREST
import MullvadSettings
import MullvadTypes
import PacketTunnelCore
import SwiftUI

struct ConnectionViewComponentPreview<Content: View>: View {
    let settingsManager = SettingsManager()
    let showIndicators: Bool
    let connectedTunnelStatus: TunnelStatus
    let disconnectedTunnelStatus = TunnelStatus(
        observedState: .disconnected,
        state: .disconnected
    )

    private static func makeConnectedTunnelStatus(obfuscationMethod: ObfuscationMethod) -> TunnelStatus {
        TunnelStatus(
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
                    isDaitaEnabled: true,
                    obfuscationMethod: obfuscationMethod
                )),
            state:
                .connected(
                    RelaySelectorStub.selectedRelays,
                    isPostQuantum: true,
                    isDaita: true
                )
        )
    }

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
        let connectedTunnelStatus = Self.makeConnectedTunnelStatus(
            obfuscationMethod: showIndicators ? .udpOverTcp : .off
        )
        self.connectedTunnelStatus = connectedTunnelStatus
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
