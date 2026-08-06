//
//  BorderStyle.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-07-24.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//
import SwiftUI

enum BorderStyle {
    case normal
    case focused
    case error
    case none

    var color: Color {
        switch self {
        case .normal:
            Color.MullvadTextField.border
        case .focused:
            Color.MullvadTextField.borderFocused
        case .error:
            Color.MullvadTextField.borderError
        case .none:
            Color.MullvadTextField.borderDisabled
        }
    }

    var lineWidth: CGFloat {
        switch self {
        case .focused, .error:
            2.0
        case .none:
            0.0
        default:
            1.0
        }
    }
}
