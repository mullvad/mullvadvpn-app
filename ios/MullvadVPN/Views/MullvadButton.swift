//
//  MullvadButton.swift
//  MullvadVPN
//
//  Created by Andrew Bulhak on 2026-07-13.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct MullvadButton: View {
    struct Style {

        /// The rank of button in the UI hierarchy.
        enum Rank {
            case primary
            case secondary
        }
        let rank: Rank
        let mainColor: Color
        let attenuatedColor: Color

        /// the primary neutral style
        static let primary = Style(
            rank: .primary, mainColor: .MullvadButton.primary, attenuatedColor: .MullvadButton.primaryPressed)
        /// the secondary neutral style
        static let secondary = Style(
            rank: .secondary, mainColor: .MullvadButton.primary, attenuatedColor: .MullvadButton.primaryPressed)
        static let destructivePrimary = Style(
            rank: .primary, mainColor: .MullvadButton.danger, attenuatedColor: .MullvadButton.dangerPressed)
        static let destructiveSecondary = Style(
            rank: .secondary, mainColor: .MullvadButton.danger, attenuatedColor: .MullvadButton.dangerPressed)
        /// the default style for a potentially destructive operation
        static let destructive = destructivePrimary
        /// the style for an operation indicating success; mainly used for the Connect button
        static let success = Style(
            rank: .primary, mainColor: .MullvadButton.positive, attenuatedColor: .MullvadButton.positivePressed)
    }

    /// An optional accessory on the leading or trailing side of the button.
    enum Accessory {
        /// An accessory containing an inert, non-tappable icon. This will be tinted in the button text colour.
        case icon(Image)
        /// An accessory showing a tappable button with an image
        case button(
            ImageResource,
            accessibilityId: AccessibilityIdentifier? = nil,
            accessibilityLabel: LocalizedStringKey? = nil,
            accessibilityHint: LocalizedStringKey? = nil,
            () -> Void)
        /// An accessory reserving space for a progress indicator, and showing it if a boolean value is true
        case progress(Bool)
    }

    /// the text to display on the main button
    var text: LocalizedStringKey
    /// The style to present this button in
    var style: Style
    /// An optional accessory to present on the leading edge of the button
    var leadingAccessory: Accessory?
    /// An optional accessory to present on the trailing edge of the button
    var trailingAccessory: Accessory?
    /// the action for when the main button is pressed
    var action: () -> Void

    @State private var mainAreaHeight: CGFloat = 0
    @State private var leadingAccessorySize: CGSize = .zero
    @State private var trailingAccessorySize: CGSize = .zero
    private let imageHeight: CGFloat = 24.0

    var body: some View {
        Button(
            action: action,
            label: {
                ZStack {
                    HStack {
                        Text(text)
                            .lineLimit(nil)
                            .multilineTextAlignment(.center)
                            .if(leadingAccessory != nil || trailingAccessory != nil) { view in
                                // Reserve space for image if present
                                view.padding(.horizontal, imageHeight)

                            }
                    }
                    HStack {
                        if let leadingAccessory {
                            accessory(leadingAccessory, position: .leading)
                                .sizeOfView { leadingAccessorySize = $0 }
                        }
                        Spacer()
                        if let trailingAccessory {
                            accessory(trailingAccessory, position: .trailing)
                                .sizeOfView { trailingAccessorySize = $0 }
                        }
                    }
                }
                .sizeOfView {
                    mainAreaHeight = $0.height
                }
            }
        ).buttonStyle(MullvadButton.ButtonStyle(style: style))
            .clipShape(Capsule())
            .accessibilityRepresentation {
                HStack {
                    leadingAccessory.map { self.accessory($0, position: .leading) }
                    Button(
                        action: action,
                        label: {
                            HStack {
                                Spacer()
                                Text(text)
                                    .lineLimit(nil)
                                    .multilineTextAlignment(.center)
                                Spacer()
                            }
                        }
                    ).buttonStyle(MullvadButton.ButtonStyle(style: style))
                        .clipShape(Capsule())
                    trailingAccessory.map { self.accessory($0, position: .trailing) }
                }
            }
    }

    @ViewBuilder
    func accessory(_ accessory: Accessory, position: TextAlignment) -> some View {
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
            .overlay {
                HStack {
                    if position == .trailing {
                        VStack { Spacer() }.frame(width: 1).background { Color.mullvadBackground }
                    }
                    Spacer()
                    if position == .leading {
                        VStack { Spacer() }.frame(width: 1).background { Color.mullvadBackground }
                    }
                }

            }
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
            MullvadButton(text: "Continue", style: .primary) {}
            MullvadButton(
                text: "Auckland", style: .primary, leadingAccessory: .button(.iconAdd, { print("+") }), action: {})
            MullvadButton(
                text: "Barcelona", style: .primary, trailingAccessory: .button(.iconChevron, { print(">") }), action: {}
            )
            MullvadButton(text: "Copenhagen", style: .secondary) {}
            MullvadButton(text: "Connect", style: .success) {}
            MullvadButton(
                text: "Gothenburg", style: .secondary, leadingAccessory: .icon(Image.mullvadIconMultihopAlways)
            ) {}
            MullvadButton(
                text: "Dublin",
                style: .secondary, trailingAccessory: .icon(Image.mullvadIconMultihopAlways)
            ) {}
            MullvadButton(
                text: "Delete", style: .destructive,
                leadingAccessory: .button(.iconCross, { print("Accessory tapped") })
            ) {}
            MullvadButton(
                text: "Log out", style: .destructiveSecondary,
                leadingAccessory: .button(.iconCross, { print("Accessory tapped") })
            ) {}
            MullvadButton(text: "Never", style: .primary, action: {}).disabled(true)
            MullvadButton(
                text: "Never", style: .secondary,
                leadingAccessory: .button(
                    .iconCross,
                    {
                        print(":-P")
                    }),
                action: {}
            ).disabled(true)
            MullvadButton(text: "Never", style: .destructive, action: {}).disabled(true)
            MullvadButton(
                text: "Connect",
                style: .primary,
                leadingAccessory: .progress(isProcessing),
                trailingAccessory: .button(
                    .iconReload,
                    {
                        print("Trailing accessory")
                    }),
                action: {
                    print("Main button 1")
                    isProcessing = true
                }
            )
        }.padding(4)
            .background(Color.mullvadBackground)
    }
}

#Preview {
    ModularButtonPreview()
}
