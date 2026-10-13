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

struct MultiplexingObfuscationSettingsView: View {
    var viewModel: any MultiplexObfuscationSettingsViewModel

    var body: some View {
        let socketString = NSLocalizedString("Socket count", comment: "")

        SingleChoiceList(
            title: socketString,
            options: [],
            value: viewModel.socketCount,
            tableAccessibilityIdentifier: AccessibilityIdentifier.wireGuardObfuscationLwoTable.asString,
            itemDescription: { item in NSLocalizedString("\(item)", comment: "") },
            parseCustomValue: {
                if let countValue = UInt8($0) {
                    viewModel.validateCount(count: countValue)
                } else {
                    nil
                }
            },
            formatCustomValue: {
                "\($0)"
            },
            customLabel: NSLocalizedString("Custom", comment: ""),
            customPrompt: NSLocalizedString("Port", comment: ""),
            customLegend: NSLocalizedString("Valid counts: >= 1", comment: ""),
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
