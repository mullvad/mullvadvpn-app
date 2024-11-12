//
//  SwitchRowView.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-11-13.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct SwitchRowView: View {
    @State private var enabled: Bool
    private let text: String

    let didToggle: (Bool) -> Void
    var didTap: (() -> Void)?

    init(
        enabled: Bool,
        text: String,
        didToggle: @escaping ((Bool) -> Void),
        didTap: (() -> Void)? = nil
    ) {
        self.enabled = enabled
        self.text = text
        self.didToggle = didToggle
        self.didTap = didTap
    }

    var body: some View {
        Toggle(isOn: $enabled, label: {
            Text(text)
        }).onChange(of: enabled, perform: { enabled in
            didToggle(enabled)
        })
        .toggleStyle(CustomToggleStyle(infoButtonAction: didTap))
        .font(.headline)
        .frame(height: UIMetrics.SettingsViewCell.height)
        .padding(UIMetrics.SettingsViewCell.layoutMargins)
        .background(Color(.primaryColor))
        .foregroundColor(Color(.primaryTextColor))
        .cornerRadius(UIMetrics.SettingsViewCell.cornerRadius)
        .accessibilityIdentifier(AccessibilityIdentifier.multihopSwitch.rawValue)
    }
}

#Preview {
    SwitchRowView(
        enabled: true,
        text: "Enable",
        didToggle: { enabled in
            print("\(enabled)")
        }, didTap: {
            print("Tapped")
        }
    )
}
