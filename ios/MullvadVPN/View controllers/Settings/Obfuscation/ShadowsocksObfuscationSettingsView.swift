//
//  ShadowsocksObfuscationSettingsView.swift
//  MullvadVPN
//
//  Created by Andrew Bulhak on 2024-11-07.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import SwiftUI

struct ShadowsocksObfuscationSettingsView<VM>: View where VM: ShadowsocksObfuscationSettingsViewModel {
    @StateObject var viewModel: VM

    var body: some View {
        let portString = NSLocalizedString(
            "SHADOWSOCKS_PORT_LABEL",
            tableName: "Shadowsocks",
            value: "Port",
            comment: ""
        )

        SingleChoiceList(
            title: portString,
            options: [WireGuardObfuscationShadowsockPort.automatic],
            value: $viewModel.value,
            itemDescription: { item in NSLocalizedString(
                "SHADOWSOCKS_PORT_VALUE_\(item)",
                tableName: "Shadowsocks",
                value: "\(item)",
                comment: ""
            ) },
            parseCustomValue: { UInt16($0).map { WireGuardObfuscationShadowsockPort.custom($0) }
            },
            formatCustomValue: {
                if case let .custom(port) = $0 {
                    "\(port)"
                } else {
                    nil
                }
            },
            customLabel: NSLocalizedString(
                "SHADOWSOCKS_PORT_VALUE_CUSTOM",
                tableName: "Shadowsocks",
                value: "Custom",
                comment: ""
            ),
            customPrompt: NSLocalizedString(
                "SHADOWSOCKS_PORT_VALUE_PORT_PROMPT",
                tableName: "Shadowsocks",
                // currently padded with spaces to make space
                value: "Port        ",
                comment: ""
            ),
            customLegend: NSLocalizedString(
                "SHADOWSOCKS_PORT_VALUE_PORT_LEGEND",
                tableName: "Shadowsocks",
                // should this be 65000 or 65535 (i.e., 0xffff)?
                value: "Valid range: 1 - 65000",
                comment: ""
            ),
            customFieldMode: .numericText
        ).onDisappear {
            viewModel.commit()
        }
    }
}

#Preview {
    var model = MockShadowsocksObfuscationSettingsViewModel(shadowsocksPort: .automatic)
    return ShadowsocksObfuscationSettingsView(viewModel: model)
}
