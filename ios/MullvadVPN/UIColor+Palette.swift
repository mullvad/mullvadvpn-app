//
//  UIColor+Palette.swift
//  MullvadVPN
//
//  Created by pronebird on 20/03/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import UIKit

extension UIColor {

    struct AccountTextField {
        struct NormalState {
            static let borderColor = UIColor(red: 0.10, green: 0.18, blue: 0.27, alpha: 1.0)
            static let textColor = UIColor(red: 0.16, green: 0.30, blue: 0.45, alpha: 1.0)
            static let backgroundColor = UIColor.white
        }

        struct ErrorState {
            static let borderColor = UIColor(red: 0.82, green: 0.01, blue: 0.11, alpha: 0.4)
            static let textColor = UIColor(red: 0.82, green: 0.01, blue: 0.11, alpha: 1.0)
            static let backgroundColor = UIColor.white
        }

        struct AuthenticatingState {
            static let borderColor = UIColor.clear
            static let textColor = UIColor.white
            static let backgroundColor = UIColor.white.withAlphaComponent(0.2)
        }
    }

    // Relay availability indicator view
    struct RelayStatusIndicator {
        static let activeColor = UIColor(red: 0.27, green: 0.68, blue: 0.30, alpha: 0.9)
        static let inactiveColor = UIColor(red: 0.82, green: 0.01, blue: 0.11, alpha: 0.95)
    }

    // Cells
    struct Cell {
        static let backgroundColor = UIColor(red: 0.16, green: 0.30, blue: 0.45, alpha: 1.0)
        static let selectedBackgroundColor = UIColor(red: 0.27, green: 0.68, blue: 0.30, alpha: 1.0)
        static let subCellBackgroundColor = UIColor(red:0.15, green:0.23, blue:0.33, alpha:1.0)
        static let subSubCellBackgroundColor = UIColor(red:0.13, green:0.20, blue:0.30, alpha:1.0)
    }
}
