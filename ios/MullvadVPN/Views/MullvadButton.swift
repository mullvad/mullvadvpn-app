//
//  MullvadButton.swift
//  MullvadVPN
//
//  Created by Andrew Bulhak on 2026-07-13.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct MullvadButton: View {
    enum Style {
        case primary
        case secondary
        case destructive
    }

    enum Accessory {
        case icon(Image)
        case button(
            ImageResource,
            accessibilityId: AccessibilityIdentifier? = nil,
            accessibilityLabel: LocalizedStringKey? = nil,
            accessibilityHint: LocalizedStringKey? = nil,
            () -> Void)
        case progress(Bool)
    }

    var text: LocalizedStringKey
    var action: () -> Void
    var leadingAccessory: Accessory?
    var trailingAccessory: Accessory?
    var style: Style

    @State private var mainAreaHeight: CGFloat = 0
    @State private var leadingAccessorySize: CGSize = .zero
    @State private var trailingAccessorySize: CGSize = .zero

    var body: some View {
        Button(
            action: action,
            label: {
                ZStack {
                    HStack {
                        if let leadingAccessory {
                            accessory(leadingAccessory)
                                .sizeOfView { leadingAccessorySize = $0 }
                        }
                        Spacer()
                        if let trailingAccessory {
                            accessory(trailingAccessory)
                                .sizeOfView { trailingAccessorySize = $0 }
                        }
                    }
                    HStack {
                        Spacer()
                        Text(text)
                        Spacer()
                    }
                }
                .sizeOfView {
                    mainAreaHeight = $0.height
                }
            }
        ).buttonStyle(MullvadButton.ButtonStyle(style: style))
            .clipShape(Capsule())
    }

    @ViewBuilder
    func accessory(_ accessory: Accessory) -> some View {
        switch accessory {
        case .icon(let image):
            image
                .renderingMode(.template)
                .resizable()
                .scaledToFit()
                .padding(10)
                .frame(
                    width: min(max(mainAreaHeight, 44), 60), height: max(mainAreaHeight, 44)
                )
        case .button(let image, let accessibilityId, let accessibilityLabel, let accessibilityHint, let action):
            Button(
                action: action,
                label: {
                    Image(image)
                        .renderingMode(.template)
                        .resizable()
                        .scaledToFit()
                        .padding(10)
                        .frame(
                            width: min(max(mainAreaHeight, 44), 60), height: max(mainAreaHeight, 44)
                        )
                }
            )
            .ifLet(accessibilityLabel) { $0.accessibilityLabel($1) }
            .ifLet(accessibilityHint) { $0.accessibilityHint($1) }
            .ifLet(accessibilityId) { $0.accessibilityIdentifier($1.asString) }
            .buttonStyle(MullvadButton.ButtonStyle(style: style, isAccessory: true))
        case .progress(let show):
            ProgressView()
                .progressViewStyle(MullvadProgressViewStyle())
                .padding(4)
                .frame(
                    width: min(max(mainAreaHeight, 44), 60), height: max(mainAreaHeight, 44)
                )
                .if(!show) { $0.hidden() }
                .background(Color.clear)
        }
    }
}

private struct ModularButtonPreview: View {
    @State var isProcessing: Bool = false
    var body: some View {
        VStack {
            MullvadButton(
                text: "Primary", action: {}, trailingAccessory: .button(.iconChevron, { print(">") }), style: .primary)
            MullvadButton(text: "Primary", action: {}, style: .primary)
            MullvadButton(text: "Secondary", action: {}, style: .secondary)
            MullvadButton(
                text: "Secondary", action: {}, leadingAccessory: .icon(Image.mullvadIconMultihopAlways),
                style: .secondary)
            MullvadButton(
                text: "Secondary", action: {}, trailingAccessory: .icon(Image.mullvadIconMultihopAlways),
                style: .secondary)
            MullvadButton(
                text: "Destructive", action: {}, leadingAccessory: .button(.iconCross, { print("Accessory tapped") }),
                style: .destructive)
            MullvadButton(text: "Disabled", action: {}, style: .primary).disabled(true)
            MullvadButton(
                text: "Disabled", action: {},
                leadingAccessory: .button(
                    .iconCross,
                    {
                        print(":-P")
                    }), style: .secondary
            ).disabled(true)
            MullvadButton(text: "Disabled", action: {}, style: .destructive).disabled(true)
            MullvadButton(
                text: "Start",
                action: {
                    print("Main button 1")
                    isProcessing = true
                },
                leadingAccessory: .progress(isProcessing),
                trailingAccessory: .button(
                    .iconReload,
                    {
                        print("Trailing accessory")
                    }),
                style: .primary)
        }.padding(4)
            .background(Color.mullvadBackground)
    }
}

#Preview {
    ModularButtonPreview()
}
