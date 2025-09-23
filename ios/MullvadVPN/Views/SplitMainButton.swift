//
//  SplitMainButton.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-12-05.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct SplitMainButton: View {
    var text: LocalizedStringKey
    var image: ImageResource
    var style: MainButtonStyle.Style
    var accessibilityId: AccessibilityIdentifier?

    @State private var secondaryButtonSize: CGSize = .zero
    @State private var primaryButtonSize: CGSize = .zero

    var primaryAction: () -> Void
    var secondaryAction: () -> Void

    var body: some View {
        HStack(spacing: 1) {
            Button(
                action: primaryAction,
                label: {
                    HStack {
                        Spacer()
                        Text(text)
                        Spacer()
                    }
                    .padding(.leading, secondaryButtonSize.width)
                    .sizeOfView { primaryButtonSize = $0 }
                }
            )
            .ifLet(accessibilityId) { view, value in
                view.accessibilityIdentifier(value.asString)
            }

            Button(
                action: secondaryAction,
                label: {
                    Image(image)
                        .resizable()
                        .scaledToFit()
                        .padding(10)
                        .frame(
                            width: min(max(primaryButtonSize.height, 44), 60), height: max(primaryButtonSize.height, 44)
                        )
                        .sizeOfView { secondaryButtonSize = $0 }
                })
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
