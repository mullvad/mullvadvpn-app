//
//  CheckboxToggleStyle.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-05-20.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct CheckboxToggleStyle: ToggleStyle {
    let accessibilityId: AccessibilityIdentifier?

    func makeBody(configuration: Configuration) -> some View {
        Button(
            action: {
                configuration.isOn.toggle()
            },
            label: {
                HStack {
                    configuration.isOn
                        ? Image.mullvadIconCheckboxSelected
                        : Image.mullvadIconCheckboxUnselected
                }
            }
        )
        .buttonStyle(PlainButtonStyle())
    }
}
