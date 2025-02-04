//
//  MainButtonStyle.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-12-05.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
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
                    ? UIColor.primaryTextColor.color
                    : UIColor.primaryTextColor.withAlphaComponent(0.2).color
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
                UIColor.primaryColor.color
            case .danger:
                UIColor.dangerColor.color
            case .success:
                UIColor.successColor.color
            }
        }

        var pressedColor: Color {
            color.darkened(by: 0.4)!
        }

        var disabledColor: Color {
            color.darkened(by: 0.6)!
        }
    }
}
