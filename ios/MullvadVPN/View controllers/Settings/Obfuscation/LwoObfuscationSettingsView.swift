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
                if let portValue = UInt16($0) {
                    validatePort(portValue)
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
            customLegend: portRangesString(for: viewModel.portRanges),
            customInputMinWidth: 100,
            customInputMaxLength: 5,
            customFieldMode: .numericText
        ).onDisappear {
            viewModel.commit()
        }
    }

    private func validatePort(_ port: UInt16) -> WireGuardObfuscationLwoPort? {
        let portIsWithinValidRanges = viewModel.portRanges
            .contains { range in
                if let minPort = range.first, let maxPort = range.last {
                    return (minPort...maxPort).contains(port)
                }
                return false
            }

        return portIsWithinValidRanges ? .custom(port) : nil
    }

    private func portRangesString(for ranges: [[UInt16]]) -> String {
        var string = "Valid ranges: "

        ranges.enumerated().forEach { (index, range) in
            if let minPort = range.first, let maxPort = range.last {
                if index != 0 {
                    string.append(", ")
                }

                string.append(String(format: "%d - %d", minPort, maxPort))
            }
        }

        return NSLocalizedString(string, comment: "")
    }
}

#Preview {
    let model = MockLwoObfuscationSettingsViewModel(lwoPort: .automatic)
    return LwoObfuscationSettingsView(viewModel: model)
}
