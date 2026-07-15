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

    func backgroundColor(for state: State) -> Color {
        switch (self, state) {
        case (.primary, .normal): Color.MullvadButton.primary
        case (.primary, .pressed): Color.MullvadButton.primaryPressed
        case (.primary, .disabled): Color.MullvadButton.primaryDisabled
        case (.secondary, _): Color.mullvadBackground
        case (.destructive, _): Color.mullvadBackground
        default: Color.blue
        }
    }

    func borderColor(for state: State) -> Color {
        switch (self, state) {
        case (.primary, _): Color.clear
        case (.secondary, .normal): Color.MullvadButton.primary
        case (.secondary, .disabled): Color.MullvadButton.primaryPressed
        case (.secondary, .pressed): Color.MullvadButton.primaryPressed
        case (.destructive, .normal): Color.MullvadButton.danger
        case (.destructive, .disabled): Color.MullvadButton.dangerPressed
        case (.destructive, .pressed): Color.MullvadButton.dangerPressed
        }
    }

}
