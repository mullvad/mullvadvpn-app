//
//  CheckBoxToggleStyle.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-01-19.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct CheckboxToggleStyle: ToggleStyle {
    var isInteractable: Bool = true

    func makeBody(configuration: Configuration) -> some View {
        Button(
            action: {
                configuration.isOn.toggle()
            },
            label: {
                HStack {
                    if isInteractable {
                        (configuration.isOn
                            ? Image(uiImage: UIImage.checkboxSelected)
                            : Image(uiImage: UIImage.checkboxUnselected))
                            .padding(8)
                    } else {
                        Image.mullvadIconTick
                    }
                    configuration.label
                        .multilineTextAlignment(.leading)
                        .font(.mullvadTiny)
                    Spacer()
                }
            }
        )
        .buttonStyle(PlainButtonStyle())
    }
}
