//
//  MullvadButtonStyle.swift
//  MullvadVPN
//
//  Created by Andrew Bulhak on 2026-07-15.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

extension MullvadButton {

    struct ButtonStyle: SwiftUI.ButtonStyle {

        var style: Style

        var isAccessory: Bool = false

        @Environment(\.isEnabled) private var isEnabled: Bool

        func makeBody(configuration: Configuration) -> some View {
            return configuration.label
                .frame(minHeight: 44)
                .foregroundStyle(
                    isEnabled
                        ? Color.mullvadTextPrimary
                        : Color.mullvadTextPrimaryDisabled
                )
                .background(
                    style.backgroundColor(for: .init(isEnabled: isEnabled, isPressed: configuration.isPressed))
                )
                .if(!isAccessory) { view in
                    view.overlay {
                        Capsule()
                            .stroke(
                                style.borderColor(for: .init(isEnabled: isEnabled, isPressed: configuration.isPressed)),
                                lineWidth: 2)
                    }

                }
                .font(.body.weight(.semibold))
        }
    }
}

extension MullvadButton.Style {
    enum State {
        case normal
        case pressed
        case disabled

        init(isEnabled: Bool, isPressed: Bool) {
            if !isEnabled {
                self = .disabled
            } else {
                self = isPressed ? .pressed : .normal
            }
        }
    }

    func activeColor(for state: State) -> Color {
        state == .normal ? mainColor : attenuatedColor
    }

    func backgroundColor(for state: State) -> Color {
        rank == .primary ? activeColor(for: state) : .mullvadBackground
    }

    func borderColor(for state: State) -> Color {
        rank == .primary ? .clear : activeColor(for: state)
    }
}
