//
//  MessageView.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-07-24.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//
import SwiftUI

struct MessageView: View {
    let message: Message
    var font: Font = .mullvadSmall
    @ScaledMetric var iconSize = 18.0

    var body: some View {
        HStack(spacing: 4.0) {
            iconView
            Text(message.text)
                .font(font)
                .foregroundStyle(message.appearance.foregroundColor)

            Spacer()
        }
    }

    @ViewBuilder
    private var iconView: some View {
        switch message.appearance.icon {
        case .none:
            EmptyView()

        case .image(let image):
            ResizableImageView(image: image, dimension: .width(iconSize))

        case .loading:
            ProgressView()
                .progressViewStyle(MullvadProgressViewStyle(size: iconSize))
        }
    }
}

extension MessageView {
    struct Message {
        let text: String
        let appearance: Appearance

        enum Icon: Equatable {
            case none
            case image(Image)
            case loading
        }

        struct Appearance {
            let foregroundColor: Color
            let icon: Icon

            static let info: Self = .init(
                foregroundColor: .mullvadTextPrimary,
                icon: .image(Image.mullvadIconInfo)
            )

            static let success: Self = .init(
                foregroundColor: .mullvadTextPrimary,
                icon: .image(Image.mullvadIconSuccess)
            )

            static let error: Self = .init(
                foregroundColor: .mullvadTextPrimary,
                icon: .image(Image.mullvadIconError)
            )

            static let loading: Self = .init(
                foregroundColor: .mullvadTextPrimary,
                icon: .loading
            )
        }
    }
}

#Preview {
    VStack {
        MessageView(
            message: .init(text: "Supporting text", appearance: .info)
        ).environment(\.dynamicTypeSize, .accessibility3)

        MessageView(message: .init(text: "Error text", appearance: .error))
        MessageView(message: .init(text: "Success text", appearance: .success))
        MessageView(message: .init(text: "Loading...", appearance: .loading))
    }
    .padding()
    .background(Color.mullvadBackground)

}
