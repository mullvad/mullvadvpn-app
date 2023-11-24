//
//  TunnelControlViewModel.swift
//  MullvadVPN
//
//  Created by Marco Nikic on 2023-11-24.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

struct TunnelControlViewModel {
    let tunnelStatus: TunnelStatus
    let secureLabelText: String
    let connectionPanel: ConnectionPanelData
    let enableButtons: Bool
    let city: String
    let country: String
    let connectedRelayName: String

    init(
        tunnelStatus: TunnelStatus,
        secureLabelText: String,
        connectionPanel: ConnectionPanelData,
        enableButtons: Bool,
        city: String,
        country: String,
        connectedRelayName: String
    ) {
        self.tunnelStatus = tunnelStatus
        self.secureLabelText = secureLabelText
        self.connectionPanel = connectionPanel
        self.enableButtons = enableButtons
        self.city = city
        self.country = country
        self.connectedRelayName = connectedRelayName
    }

    func update(status: TunnelStatus) -> TunnelControlViewModel {
        TunnelControlViewModel(
            tunnelStatus: status,
            secureLabelText: secureLabelText,
            connectionPanel: connectionPanel,
            enableButtons: enableButtons,
            city: city,
            country: country,
            connectedRelayName: connectedRelayName
        )
    }

    func update(outgoingConnectionInfo: OutgoingConnectionInfo) -> TunnelControlViewModel {
        let inPort = tunnelStatus.observedState.connectionState?.remotePort ?? 0

        var connectionPanelData = ConnectionPanelData(inAddress: "")
        if let tunnelRelay = tunnelStatus.state.relay {
            var protocolLayer = ""
            if case let .connected(state) = tunnelStatus.observedState {
                protocolLayer = state.transportLayer == .tcp ? "TCP" : "UDP"
            }

            connectionPanelData = ConnectionPanelData(
                inAddress: "\(tunnelRelay.endpoint.ipv4Relay.ip):\(inPort) \(protocolLayer)",
                outAddress: outgoingConnectionInfo.outAddress
            )
        }

        return TunnelControlViewModel(
            tunnelStatus: tunnelStatus,
            secureLabelText: secureLabelText,
            connectionPanel: connectionPanelData,
            enableButtons: enableButtons,
            city: city,
            country: country,
            connectedRelayName: connectedRelayName
        )
    }

    static var empty: Self {
        TunnelControlViewModel(
            tunnelStatus: TunnelStatus(),
            secureLabelText: "",
            connectionPanel: ConnectionPanelData(inAddress: ""),
            enableButtons: true,
            city: "",
            country: "",
            connectedRelayName: ""
        )
    }
}
