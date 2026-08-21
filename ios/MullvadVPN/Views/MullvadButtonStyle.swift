// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import SwiftUI

extension MullvadButton {

    struct ButtonStyle: SwiftUI.ButtonStyle {

        var style: Style

        var accessoryPosition: TextAlignment? = nil

        @Environment(\.isEnabled) private var isEnabled: Bool

        func makeBody(configuration: Configuration) -> some View {
            let borderInset: CGFloat = accessoryPosition != nil ? 0 : 0
            let backgroundInsets = EdgeInsets(
                top: borderInset,
                leading: (accessoryPosition == .trailing) ? -.infinity : borderInset,
                bottom: borderInset,
                trailing: (accessoryPosition == .leading) ? -.infinity : borderInset)
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
                .overlay {
                    Capsule()
                        .stroke(
                            style.borderColor(for: .init(isEnabled: isEnabled, isPressed: configuration.isPressed)),
                            lineWidth: 2
                        )
                        .padding(backgroundInsets)
                        .clipped()
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
