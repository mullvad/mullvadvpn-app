//
//  ShadowsocksObfuscationSettingsView.swift
//  MullvadVPN
//
//  Created by Andrew Bulhak on 2024-11-07.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import SwiftUI

struct ShadowsocksObfuscationSettingsView<VM>: View where VM: ShadowsocksObfuscationSettingsViewModel {
    @StateObject var viewModel: VM

    var body: some View {
        let portString = NSLocalizedString(
            "SHADOWSOCKS_PORT_LABEL",
            value: "Port",
            comment: ""
        )

        SingleChoiceList(
            title: portString,
            options: [WireGuardObfuscationShadowsocksPort.automatic],
            value: $viewModel.value,
            tableAccessibilityIdentifier: AccessibilityIdentifier.wireGuardObfuscationShadowsocksTable.asString,
            itemDescription: { item in NSLocalizedString(
                "SHADOWSOCKS_PORT_VALUE_\(item)",
                value: "\(item)",
                comment: ""
            ) },
            parseCustomValue: { UInt16($0).flatMap { $0 > 0 ? WireGuardObfuscationShadowsocksPort.custom($0) : nil }
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
                value: "Custom",
                comment: ""
            ),
            customPrompt: NSLocalizedString(
                "SHADOWSOCKS_PORT_VALUE_PORT_PROMPT",
                value: "Port",
                comment: ""
            ),
            customLegend: NSLocalizedString(
                "SHADOWSOCKS_PORT_VALUE_PORT_LEGEND",
                value: "Valid range: 1 - 65535",
                comment: ""
            ),
            customInputMinWidth: 100,
            customInputMaxLength: 5,
            customFieldMode: .numericText
        ).onDisappear {
            viewModel.commit()
        }
    }
}

#Preview {
    let model = MockShadowsocksObfuscationSettingsViewModel(shadowsocksPort: .automatic)
    return ShadowsocksObfuscationSettingsView(viewModel: model)
}
