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
    let enableButtons: Bool
    let city: String
    let country: String
    let connectedRelaysName: String
    let outgoingConnectionInfo: OutgoingConnectionInfo?

    var connectionPanel: ConnectionPanelData? {
        guard let tunnelRelays = tunnelStatus.state.relays else {
            return nil
        }

        var portAndTransport = ""
        if let inPort = tunnelStatus.observedState.connectionState?.remotePort {
            let protocolLayer = tunnelStatus.observedState.connectionState?.transportLayer == .tcp ? "TCP" : "UDP"
            portAndTransport = ":\(inPort) \(protocolLayer)"
        }

        return ConnectionPanelData(
            inAddress: "\(tunnelRelays.entry?.endpoint.ipv4Relay.ip ?? tunnelRelays.exit.endpoint.ipv4Relay.ip)\(portAndTransport)",
            outAddress: outgoingConnectionInfo?.outAddress
        )
    }

    static var empty: Self {
        TunnelControlViewModel(
            tunnelStatus: TunnelStatus(),
            secureLabelText: "",
            enableButtons: true,
            city: "",
            country: "",
            connectedRelaysName: "",
            outgoingConnectionInfo: nil
        )
    }

    func update(status: TunnelStatus) -> TunnelControlViewModel {
        TunnelControlViewModel(
            tunnelStatus: status,
            secureLabelText: secureLabelText,
            enableButtons: enableButtons,
            city: city,
            country: country,
            connectedRelaysName: connectedRelaysName,
            outgoingConnectionInfo: nil
        )
    }

    func update(outgoingConnectionInfo: OutgoingConnectionInfo) -> TunnelControlViewModel {
        TunnelControlViewModel(
            tunnelStatus: tunnelStatus,
            secureLabelText: secureLabelText,
            enableButtons: enableButtons,
            city: city,
            country: country,
            connectedRelaysName: connectedRelaysName,
            outgoingConnectionInfo: outgoingConnectionInfo
        )
    }
}
