//
//  MessageView.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-07-24.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//
import SwiftUI

struct Message: Equatable {
    let text: String
    let appearance: Appearance

    struct Appearance: Equatable {
        let foregroundColor: Color
        let icon: Image?

        static let info: Self = .init(
            foregroundColor: .mullvadTextPrimary,
            icon: .mullvadIconInfo
        )

        static let success: Self = .init(
            foregroundColor: .mullvadTextPrimary,
            icon: .mullvadIconSuccess
        )

        static let error: Self = .init(
            foregroundColor: .mullvadDangerColor,
            icon: .mullvadIconError
        )
    }
}

struct MessageView: View {
    let message: Message
    var font: Font = .mullvadSmall

    var body: some View {
        HStack(spacing: 4.0) {
            if let image = message.appearance.icon {
                ResizableImageView(image: image, dimension: .width(18.0))
            }
            Text(message.text)
                .font(font)
                .foregroundStyle(message.appearance.foregroundColor)

            Spacer()
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
    }
    .padding()
    .background(Color.mullvadBackground)

}
