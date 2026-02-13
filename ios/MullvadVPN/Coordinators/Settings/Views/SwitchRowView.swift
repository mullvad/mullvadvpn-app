//
//  SwitchRowView.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-11-13.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

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
