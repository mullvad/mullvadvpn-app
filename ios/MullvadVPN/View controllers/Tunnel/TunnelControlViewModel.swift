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

    init(from other: TunnelControlViewModel) {
        self.init(
            tunnelStatus: other.tunnelStatus,
            secureLabelText: other.secureLabelText,
            connectionPanel: other.connectionPanel,
            enableButtons: other.enableButtons,
            city: other.city,
            country: other.country,
            connectedRelayName: other.connectedRelayName
        )
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
        TunnelControlViewModel(
            tunnelStatus: tunnelStatus,
            secureLabelText: secureLabelText,
            connectionPanel: ConnectionPanelData(
                inAddress: "\(tunnelStatus.state.relay?.endpoint.ipv4Relay.description ?? "no info")",
                outAddress: outgoingConnectionInfo.outAddress
            ),
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
