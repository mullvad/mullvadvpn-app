// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadSettings
import SwiftUI

struct LwoObfuscationSettingsView: View {
    var viewModel: any LwoObfuscationSettingsViewModel

    var body: some View {
        let portString = NSLocalizedString("Port", comment: "")

        SingleChoiceList(
            title: portString,
            options: [WireGuardObfuscationLwoPort.automatic],
            value: viewModel.port,
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
    let model = MockLwoObfuscationSettingsViewModel(port: $port)
    LwoObfuscationSettingsView(viewModel: model)
}
