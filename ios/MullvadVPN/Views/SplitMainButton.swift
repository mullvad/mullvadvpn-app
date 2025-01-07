//
//  SplitMainButton.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-12-05.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct SplitMainButton: View {
    var text: LocalizedStringKey
    var image: ImageResource
    var style: MainButtonStyle.Style
    var accessibilityId: AccessibilityIdentifier?

    @State private var secondaryButtonWidth: CGFloat = 0

    var primaryAction: () -> Void
    var secondaryAction: () -> Void

    var body: some View {
        HStack(spacing: 1) {
            Button(action: primaryAction, label: {
                HStack {
                    Spacer()
                    Text(text)
                    Spacer()
                }
                .padding(.trailing, -secondaryButtonWidth)
            })
            .ifLet(accessibilityId) { view, value in
                view.accessibilityIdentifier(value.asString)
            }

            Button(action: secondaryAction, label: {
                Image(image)
                    .resizable()
                    .scaledToFit()
                    .frame(width: 24, height: 24)
                    .padding(10)
            })
            .aspectRatio(1, contentMode: .fit)
            .sizeOfView { secondaryButtonWidth = $0.width }
        }
        .buttonStyle(MainButtonStyle(style))
        .cornerRadius(UIMetrics.MainButton.cornerRadius)
    }
}

#Preview {
    SplitMainButton(
        text: "Select location",
        image: .iconReload,
        style: .default,
        primaryAction: {
            print("Tapped primary")
        },
        secondaryAction: {
            print("Tapped secondary")
        }
    )
}
