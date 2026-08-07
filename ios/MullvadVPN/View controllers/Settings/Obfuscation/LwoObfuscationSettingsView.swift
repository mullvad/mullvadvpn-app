//
//  LwoObfuscationSettingsView.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-02-02.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import SwiftUI

struct LwoObfuscationSettingsView: View {
    var viewModel: any LwoObfuscationSettingsViewModel
    var port: Binding<WireGuardObfuscationLwoPort>

    var body: some View {
        let portString = NSLocalizedString("Port", comment: "")

        SingleChoiceList(
            title: portString,
            options: [WireGuardObfuscationLwoPort.automatic],
            value: port,
            tableAccessibilityIdentifier: AccessibilityIdentifier.wireGuardObfuscationLwoTable.asString,
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
    @Previewable @State var port = WireGuardObfuscationLwoPort.automatic
    let model = MockLwoObfuscationSettingsViewModel(lwoPort: .automatic)
    LwoObfuscationSettingsView(viewModel: model, port: $port)
}
