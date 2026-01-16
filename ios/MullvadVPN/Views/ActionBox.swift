//
//  ActionBox.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-01-19.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct ActionBox<Style: ToggleStyle>: View {
    @State var isChecked: Bool

    let label: String
    let toggleStyle: Style

    var didToggle: (Bool) -> Void

    var body: some View {
        Toggle(NSLocalizedString(label, comment: ""), isOn: $isChecked)
            .padding(UIMetrics.ActionBox.padding)
            .border(Color.MullvadActionBox.border)
            .cornerRadius(UIMetrics.ActionBox.cornerRadius)
            .toggleStyle(toggleStyle)
            .onChange(of: isChecked) { _, newValue in
                didToggle(newValue)
            }
    }
}
