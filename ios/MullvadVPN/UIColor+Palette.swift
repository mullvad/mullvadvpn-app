//
//  UIColor+Palette.swift
//  MullvadVPN
//
//  Created by pronebird on 20/03/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import UIKit

extension UIColor {

    enum AccountTextField {
        enum NormalState {
            static let borderColor = UIColor(red: 0.10, green: 0.18, blue: 0.27, alpha: 1.0)
            static let textColor = UIColor(red: 0.16, green: 0.30, blue: 0.45, alpha: 1.0)
            static let backgroundColor = UIColor.white
        }

        enum ErrorState {
            static let borderColor = dangerColor.withAlphaComponent(0.4)
            static let textColor = dangerColor
            static let backgroundColor = UIColor.white
        }

        enum AuthenticatingState {
            static let borderColor = UIColor.clear
            static let textColor = UIColor.white
            static let backgroundColor = UIColor.white.withAlphaComponent(0.2)
        }
    }

    // Relay availability indicator view
    enum RelayStatusIndicator {
        static let activeColor = successColor.withAlphaComponent(0.9)
        static let inactiveColor = dangerColor.withAlphaComponent(0.95)
    }

    // Cells
    enum Cell {
        static let backgroundColor = UIColor(red: 0.16, green: 0.30, blue: 0.45, alpha: 1.0)
        static let disabledBackgroundColor = backgroundColor.darkened(by: 0.3)!

        static let selectedBackgroundColor = successColor
        static let disabledSelectedBackgroundColor = selectedBackgroundColor.darkened(by: 0.3)!

        static let selectedAltBackgroundColor = backgroundColor.darkened(by: 0.2)!
    }

    enum SubCell {
        static let backgroundColor = UIColor(red: 0.15, green: 0.23, blue: 0.33, alpha: 1.0)
        static let disabledBackgroundColor = backgroundColor.darkened(by: 0.3)!
    }

    enum SubSubCell {
        static let backgroundColor = UIColor(red: 0.13, green: 0.20, blue: 0.30, alpha: 1.0)
        static let disabledBackgroundColor = backgroundColor.darkened(by: 0.3)!
    }

    enum HeaderBar {
        static let defaultBackgroundColor = UIColor(red: 0.16, green: 0.30, blue: 0.45, alpha: 1.0)
        static let unsecuredBackgroundColor = dangerColor
        static let securedBackgroundColor = successColor
    }

    // Common colors
    static let dangerColor = UIColor(red: 0.89, green: 0.25, blue: 0.22, alpha: 1.0)
    static let successColor = UIColor(red: 0.27, green: 0.68, blue: 0.30, alpha: 1.0)
}
