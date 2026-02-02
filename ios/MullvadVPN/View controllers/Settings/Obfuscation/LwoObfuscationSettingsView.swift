//
//  LwoObfuscationSettingsView.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-02-02.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import SwiftUI

struct LwoObfuscationSettingsView<VM>: View where VM: LwoObfuscationSettingsViewModel {
    @StateObject var viewModel: VM

    var body: some View {
        let portString = NSLocalizedString("Port", comment: "")

        SingleChoiceList(
            title: portString,
            options: [WireGuardObfuscationLwoPort.automatic],
            value: $viewModel.value,
            tableAccessibilityIdentifier: AccessibilityIdentifier.wireGuardObfuscationLwoTable.asString,
            itemDescription: { item in NSLocalizedString("\(item)", comment: "") },
            parseCustomValue: {
                UInt16($0).flatMap { $0 > 0 ? WireGuardObfuscationLwoPort.custom($0) : nil }
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
            customLegend: String(
                format: NSLocalizedString("Valid range: %d - %d", comment: ""), arguments: [1, 65535]),
            customInputMinWidth: 100,
            customInputMaxLength: 5,
            customFieldMode: .numericText
        ).onDisappear {
            viewModel.commit()
        }
    }
}

#Preview {
    let model = MockLwoObfuscationSettingsViewModel(lwoPort: .automatic)
    return LwoObfuscationSettingsView(viewModel: model)
}
