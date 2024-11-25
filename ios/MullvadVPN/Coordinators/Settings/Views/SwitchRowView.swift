//
//  SwitchRowView.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-11-13.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct SwitchRowView: View {
    @Binding var enabled: Bool
    let text: String

    var didTapInfoButton: (() -> Void)?

    var body: some View {
        Toggle(isOn: $enabled, label: {
            Text(text)
        })
        .toggleStyle(CustomToggleStyle(infoButtonAction: didTapInfoButton))
        .font(.headline)
        .frame(height: UIMetrics.SettingsRowView.height)
        .padding(UIMetrics.SettingsRowView.layoutMargins)
        .background(Color(.primaryColor))
        .foregroundColor(Color(.primaryTextColor))
        .cornerRadius(UIMetrics.SettingsRowView.cornerRadius)
    }
}

#Preview("SwitchRowView") {
    StatefulPreviewWrapper(true) {
        SwitchRowView(
            enabled: $0,
            text: "Enable",
            didTapInfoButton: {
                print("Tapped")
            }
        )
    }
}
