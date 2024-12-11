//
//  MainButtonStyle.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-12-05.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct MainButtonStyle: ButtonStyle {
    @State var style: Style

    init(_ style: Style) {
        self.style = style
    }

    func makeBody(configuration: Configuration) -> some View {
        configuration.label
            .padding(.horizontal, 8)
            .frame(height: 44)
            .foregroundColor(
                configuration.isPressed
                    ? UIColor.secondaryTextColor.color
                    : UIColor.primaryTextColor.color
            )
            .background(style.color)
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
