//
//  UDPOverTCPObfuscationSettingsView.swift
//  MullvadVPN
//
//  Created by Andrew Bulhak on 2024-10-28.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import SwiftUI

struct UDPOverTCPObfuscationSettingsView: View {
    var port: Binding<WireGuardObfuscationUdpOverTcpPort>

    var body: some View {
        let portString = NSLocalizedString("Port", comment: "")
        SingleChoiceList(
            title: portString,
            options: [WireGuardObfuscationUdpOverTcpPort.automatic, .port80, .port443, .port5001],
            value: port,
            tableAccessibilityIdentifier: AccessibilityIdentifier.wireGuardObfuscationUdpOverTcpTable.asString,
            itemDescription: { item in
                "\(item)"
            }
        )
    }
}

#Preview {
    @Previewable @State var port = WireGuardObfuscationUdpOverTcpPort.port5001
    return UDPOverTCPObfuscationSettingsView(port: $port)
}
