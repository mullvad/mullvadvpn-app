//
//  ModularButton.swift
//  MullvadVPN
//
//  Created by Andrew Bulhak on 2026-07-13.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct ModularButton: View {
    enum Accessory {
        case image(Image)
        case button(ImageResource, () -> Void)
        case progress(Bool)
    }

    struct Segment {
        let text: String
        let action: () -> Void
    }

    var segments: [Segment]
    var leadingAccessory: Accessory?
    var trailingAccessory: Accessory?
    var style: MainButtonStyle.Style

    @State private var mainAreaHeight: CGFloat = 0
    @State private var leadingAccessorySize: CGSize = .zero
    @State private var trailingAccessorySize: CGSize = .zero

    var body: some View {
        HStack(spacing: 1) {
            if let leadingAccessory {
                accessory(leadingAccessory)
                    .sizeOfView { leadingAccessorySize = $0 }
            }
            // Limitation: duplicate segment names are not allowed.
            ForEach(segments, id: \.text) { segment in
                Button(
                    action: segment.action,
                    label: {
                        HStack {
                            Spacer()
                            Text(segment.text)
                            Spacer()
                        }
                        .sizeOfView {
                            mainAreaHeight = $0.height
                        }
                    }
                )
            }
            if let trailingAccessory {
                accessory(trailingAccessory)
                    .sizeOfView { trailingAccessorySize = $0 }
            }
        }.buttonStyle(MainButtonStyle(style))
            .cornerRadius(mainAreaHeight)
    }

    @ViewBuilder
    func accessory(_ accessory: Accessory) -> some View {
        switch accessory {
        case .image(let image):
            ResizableImageView(image: image, dimension: .height(24))  // FIXME
        case .button(let image, let action):
            Button(
                action: action,
                label: {
                    Image(image)
                        .resizable()
                        .scaledToFit()
                        .padding(10)
                        .frame(
                            width: min(max(mainAreaHeight, 44), 60), height: max(mainAreaHeight, 44)
                        )
                }
            )
        case .progress(let show):
            ProgressView()
                .progressViewStyle(MullvadProgressViewStyle())
                .padding(4)
                .frame(
                    width: min(max(mainAreaHeight, 44), 60), height: max(mainAreaHeight, 44)
                )
                .if(!show) { $0.hidden() }
                .background(style.color)
        }
    }
}

private struct ModularButtonPreview: View {
    @State var isProcessing: Bool = false
    var body: some View {
        VStack {
            ModularButton(
                segments: [
                    .init(
                        text: "One",
                        action: {
                            print("Main button 1")
                            isProcessing = true
                        }),
                    .init(
                        text: "Two",
                        action: {
                            print("Main button 2")
                        }),
                ],
                leadingAccessory: .progress(isProcessing),
                trailingAccessory: .button(
                    .iconReload,
                    {
                        print("Trailing accessory")
                    }),
                style: .default)
        }.padding(4)
    }
}

#Preview {
    ModularButtonPreview()
}
