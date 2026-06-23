//
//  MainButton.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-12-04.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
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

    private let imageHeight: CGFloat = 24.0

    var body: some View {
        Button(
            action: action,
            label: {
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
                            ResizableImageView(image: image, dimension: .height(imageHeight))
                            Spacer()
                        }
                        Spacer()
                        if imagePosition == .trailing, let image = image {
                            Spacer()  // Push the text to center
                            ResizableImageView(image: image, dimension: .height(imageHeight))
                        }
                    }
                }
            }
        )
        .buttonStyle(MainButtonStyle(style))
        .cornerRadius(UIMetrics.MainButton.cornerRadius)
    }
}

#Preview {
    MainButton(text: "Connect", style: .success) {
        print("Tapped")
    }
}
