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
    var disabled = false

    var primaryAction: () -> Void
    var secondaryAction: () -> Void

    @State private var width: CGFloat = 0

    var body: some View {
        HStack(spacing: 1) {
            Button(action: primaryAction, label: {
                HStack {
                    Spacer()
                    Text(text)
                    Spacer()
                }
                .padding(.trailing, -width)
            })
            Button(action: secondaryAction, label: {
                Image(image)
                    .resizable()
                    .scaledToFit()
                    .padding(4)
            })
            .aspectRatio(1, contentMode: .fit)
            .sizeOfView { width = $0.width }
        }
        .buttonStyle(MainButtonStyle(style, disabled: disabled))
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
