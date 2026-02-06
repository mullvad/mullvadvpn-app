//
//  MainButtonStyle.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-12-05.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct MainButtonStyle: ButtonStyle {
    var style: Style
    @Environment(\.isEnabled) private var isEnabled: Bool

    init(_ style: Style) {
        self.style = style
    }

    func makeBody(configuration: Configuration) -> some View {
        return configuration.label
            .frame(minHeight: 44)
            .foregroundColor(
                isEnabled
                    ? .mullvadTextPrimary
                    : .mullvadTextPrimaryDisabled
            )
            .background(
                isEnabled
                    ? configuration.isPressed
                        ? style.pressedColor
                        : style.color
                    : style.disabledColor
            )
            .font(.body.weight(.semibold))
    }
}

extension MainButtonStyle {
    enum Style {
        case `default`
        case danger
        case success

        var color: Color {
            switch self {
            case .default:
                Color.MullvadButton.primary
            case .danger:
                Color.MullvadButton.danger
            case .success:
                Color.MullvadButton.positive
            }
        }

        var pressedColor: Color {
            switch self {
            case .default:
                Color.MullvadButton.primaryPressed
            case .danger:
                Color.MullvadButton.dangerPressed
            case .success:
                Color.MullvadButton.positivePressed
            }
        }

        var disabledColor: Color {
            switch self {
            case .default:
                Color.MullvadButton.primaryDisabled
            case .danger:
                Color.MullvadButton.dangerDisabled
            case .success:
                Color.MullvadButton.positiveDisabled
            }
        }
    }
}
