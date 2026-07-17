//
//  WireGuardPortSettingsView.swift
//  MullvadVPN
//
//  Created by Marco Nikic on 2026-07-28.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import SwiftUI

struct WireGuardPortSettingsView: View {
    var viewModel: any WireGuardPortSettingsViewModel
    let options: [WireGuardPort]

    var body: some View {
        let portString = NSLocalizedString("Port", comment: "")

        SingleChoiceList(
            title: portString,
            options: options,
            value: viewModel.port,
            tableAccessibilityIdentifier: AccessibilityIdentifier.wireGuardObfuscationPortTable.asString,
            itemDescription: { item in NSLocalizedString("\(item)", comment: "") },
            parseCustomValue: {
                if let portValue = UInt16($0) {
                    viewModel.validatePort(portValue)
                } else {
                    nil
                }
            },
            formatCustomValue: {
                if case let .custom(port) = $0 {
                    "\(port)"
                } else {
                    nil
                }
            },
            customLabel: NSLocalizedString("Custom", comment: ""),
            customPrompt: NSLocalizedString("Port", comment: ""),
            customLegend: viewModel.portRangesString(),
            customInputMinWidth: 100,
            customInputMaxLength: 5,
            customFieldMode: .numericText
        )
    }
}

#Preview {
    @Previewable @State var port = WireGuardPort.automatic
    let model = WireGuardPortSettingsViewModelStub(port: $port)
    WireGuardPortSettingsView(viewModel: model, options: [.automatic, .port51820, .port53])
}
