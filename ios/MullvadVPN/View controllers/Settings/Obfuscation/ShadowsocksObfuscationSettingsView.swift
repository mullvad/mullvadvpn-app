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

struct ShadowsocksObfuscationSettingsView: View {
    var port: Binding<WireGuardObfuscationShadowsocksPort>

    var body: some View {
        let portString = NSLocalizedString("Port", comment: "")

        SingleChoiceList(
            title: portString,
            options: [WireGuardObfuscationShadowsocksPort.automatic],
            value: port,
            tableAccessibilityIdentifier: AccessibilityIdentifier.wireGuardObfuscationShadowsocksTable.asString,
            itemDescription: { item in NSLocalizedString("\(item)", comment: "") },
            parseCustomValue: {
                UInt16($0).flatMap { $0 > 0 ? WireGuardObfuscationShadowsocksPort.custom($0) : nil }
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
        )
    }
}

#Preview {
    @Previewable @State var port = WireGuardObfuscationShadowsocksPort.automatic
    return ShadowsocksObfuscationSettingsView(port: $port)
}
