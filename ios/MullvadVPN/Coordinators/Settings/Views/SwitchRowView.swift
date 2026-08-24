// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import SwiftUI

struct SwitchRowView: View {
    @Binding var isOn: Bool

    var disabled = false
    let text: String
    var accessibilityId: AccessibilityIdentifier?

    var didTapInfoButton: (() -> Void)?

    var body: some View {
        HStack {
            Text(text)
                .opacity(disabled ? 0.2 : 1)

            if let didTapInfoButton {
                Button(action: didTapInfoButton) {
                    Image(.iconInfo)
                }
                .adjustingTapAreaSize()
                .tint(.white)
            }

            Spacer()
            Toggle(
                isOn: $isOn,
                label: {
                    Text(text)
                }
            )
            .toggleStyle(
                CustomToggleStyle(
                    disabled: disabled,
                    accessibilityId: accessibilityId,
                    infoButtonAction: didTapInfoButton
                )
            )
            .disabled(disabled)
        }
        .font(.mullvadSmall)
        .padding(UIMetrics.SettingsRowView.layoutMargins)
        .background(Color(.primaryColor))
        .foregroundColor(Color(.primaryTextColor))
        .cornerRadius(UIMetrics.SettingsRowView.cornerRadius)
    }
}

#Preview("SwitchRowView") {
    StatefulPreviewWrapper(true) {
        SwitchRowView(
            isOn: $0,
            text: "Enable",
            didTapInfoButton: {
                print("Tapped")
            }
        )
    }
}
