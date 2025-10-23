//
//  UDPOverTCPObfuscationSettingsView.swift
//  MullvadVPN
//
//  Created by Andrew Bulhak on 2024-10-28.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import SwiftUI

struct UDPOverTCPObfuscationSettingsView<VM>: View where VM: UDPOverTCPObfuscationSettingsViewModel {
    @StateObject var viewModel: VM

    var body: some View {
        let portString = NSLocalizedString("Port", comment: "")
        SingleChoiceList(
            title: portString,
            options: [WireGuardObfuscationUdpOverTcpPort.automatic, .port80, .port5001],
            value: $viewModel.value,
            tableAccessibilityIdentifier: AccessibilityIdentifier.wireGuardObfuscationUdpOverTcpTable.asString,
            itemDescription: { item in
                "\(item)"
            }
        ).onDisappear {
            viewModel.commit()
        }
    }
}

#Preview {
    let model = MockUDPOverTCPObfuscationSettingsViewModel(udpTcpPort: .port5001)
    return UDPOverTCPObfuscationSettingsView(viewModel: model)
}
