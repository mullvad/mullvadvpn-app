//
//  CustomToggle.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-11-13.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

/// Custom (default) toggle style used for switches.
struct CustomToggleStyle: ToggleStyle {
    private let width: CGFloat = 48
    private let height: CGFloat = 30
    private let circleRadius: CGFloat = 23

    var disabled = false
    var infoButtonAction: (() -> Void)?

    func makeBody(configuration: Configuration) -> some View {
        HStack {
            configuration.label
                .opacity(disabled ? 0.2 : 1)

            if let infoButtonAction {
                Button(action: infoButtonAction) {
                    Image(.iconInfo)
                }
                .adjustingTapAreaSize()
                .tint(.white)
            }

            Spacer()

            ZStack(alignment: configuration.isOn ? .trailing : .leading) {
                Capsule(style: .circular)
                    .frame(width: width, height: height)
                    .foregroundColor(.clear)
                    .overlay(
                        RoundedRectangle(cornerRadius: circleRadius)
                            .stroke(
                                Color(.white.withAlphaComponent(0.8)),
                                lineWidth: 2
                            )
                    )
                    .opacity(disabled ? 0.2 : 1)

                Circle()
                    .frame(width: circleRadius, height: circleRadius)
                    .padding(3)
                    .foregroundColor(
                        configuration.isOn
                            ? Color(uiColor: UIColor.Switch.onThumbColor)
                            : Color(uiColor: UIColor.Switch.offThumbColor)
                    )
                    .opacity(disabled ? 0.4 : 1)
            }
            .onTapGesture {
                if !disabled {
                    toggle(configuration)
                }
            }
            .adjustingTapAreaSize()
        }
    }

    private func toggle(_ configuration: Configuration) {
        withAnimation(.easeInOut(duration: 0.25)) {
            configuration.$isOn.wrappedValue.toggle()
        }
    }
}
