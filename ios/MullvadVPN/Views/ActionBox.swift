//
//  ActionBox.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-01-19.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct ActionBox: View {
    @State var isOn: Bool

    let label: String
    var didToggle: (Bool) -> Void

    var body: some View {
        HStack {
            Toggle(NSLocalizedString(label, comment: ""), isOn: $isOn)
                .padding(UIMetrics.ActionBox.padding)
                .border(Color.MullvadActionBox.border)
                .cornerRadius(UIMetrics.ActionBox.cornerRadius)
                .toggleStyle(CheckboxToggleStyle())
                .onChange(of: isOn) { _, newValue in
                    didToggle(newValue)
                }
        }
    }
}
