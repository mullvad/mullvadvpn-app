//
//  MainButton.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-12-04.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import SwiftUI
enum MainButtonImagePosition {
    case leading
    case trailing
}

struct MainButton: View {
    var text: LocalizedStringKey
    var style: MainButtonStyle.Style

    var image: Image?
    var imagePosition: MainButtonImagePosition = .leading
    var action: () -> Void

    @State private var imageHeight: CGFloat = 24.0

    var body: some View {
        Button(action: action, label: {
            ZStack {
                // Centered Text
                Text(text)
                    .lineLimit(nil)
                    .multilineTextAlignment(.center)
                    .if(image != nil) { view in
                        // Reserve space for image if present
                        view.padding(.horizontal, imageHeight)
                    }

                // Image on Leading or Trailing
                HStack {
                    if imagePosition == .leading, let image = image {
                        image
                            .resizable()
                            .scaledToFit()
                            .frame(height: imageHeight)
                            .padding(.leading, 8.0)
                        Spacer()
                    }
                    Spacer()
                    if imagePosition == .trailing, let image = image {
                        Spacer() // Push the text to center
                        image
                            .resizable()
                            .scaledToFit()
                            .frame(height: imageHeight)
                            .padding(.trailing, 8.0)
                    }
                }
            }
        })
        .sizeOfView { size in
            let actualHeight = size.height - 16.0
            let baseHeight = max(actualHeight, 24.0)
            imageHeight = baseHeight * 0.8
        }
        .buttonStyle(MainButtonStyle(style))
        .cornerRadius(UIMetrics.MainButton.cornerRadius)
    }
}

#Preview {
    MainButton(text: "Connect", style: .success) {
        print("Tapped")
    }
}
