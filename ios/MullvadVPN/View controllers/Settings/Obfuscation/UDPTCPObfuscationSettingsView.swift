//
//  UDPTCPObfuscationSettingsView.swift
//  MullvadVPN
//
//  Created by Andrew Bulhak on 2024-10-28.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import SwiftUI

struct UDPTCPObfuscationSettingsView<VM>: View where VM: UDPTCPObfuscationSettingsViewModel {
    @StateObject var viewModel: VM

    var body: some View {
        let portString = NSLocalizedString(
            "UDP_TCP_PORT_LABEL",
            tableName: "UdpToTcp",
            value: "Port",
            comment: ""
        )
        SingleChoiceList(
            title: portString,
            options: [WireGuardObfuscationUdpOverTcpPort.automatic, .port80, .port5001],
            value: $viewModel.value,
            itemDescription: { item in NSLocalizedString(
                "UDP_TCP_PORT_VALUE_\(item)",
                tableName: "UdpToTcp",
                value: "\(item)",
                comment: ""
            ) }
        )
    }
}

#Preview {
    let model = MockUDPTCPObfuscationSettingsViewModel(udpTcpPort: .port5001)
    return UDPTCPObfuscationSettingsView(viewModel: model)
}
