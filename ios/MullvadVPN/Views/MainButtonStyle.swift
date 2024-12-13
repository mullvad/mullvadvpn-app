//
//  MainButtonStyle.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-12-05.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct MainButtonStyle: ButtonStyle {
    var style: Style
    @State var disabled: Bool

    init(_ style: Style, disabled: Bool = false) {
        self.style = style
        self.disabled = disabled
    }

    func makeBody(configuration: Configuration) -> some View {
        configuration.label
            .padding(.horizontal, 8)
            .frame(height: 44)
            .foregroundColor(
                configuration.isPressed
                    ? UIColor.secondaryTextColor.color
                    : disabled
                        ? UIColor.primaryTextColor.withAlphaComponent(0.2).color
                        : UIColor.primaryTextColor.color
            )
            .background(
                disabled
                    ? style.color.darkened(by: 0.6)
                    : style.color
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
                Color(UIColor.primaryColor)
            case .danger:
                Color(UIColor.dangerColor)
            case .success:
                Color(UIColor.successColor)
            }
        }
    }
}
