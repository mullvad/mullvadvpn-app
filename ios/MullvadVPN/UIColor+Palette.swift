//
//  UIColor+Palette.swift
//  MullvadVPN
//
//  Created by pronebird on 20/03/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import UIKit

extension UIColor {
    enum AccountTextField {
        enum NormalState {
            static let borderColor = secondaryColor
            static let textColor = primaryColor
            static let backgroundColor = UIColor.white
        }

        enum ErrorState {
            static let borderColor = dangerColor.withAlphaComponent(0.4)
            static let textColor = dangerColor
            static let backgroundColor = UIColor.white
        }

        enum AuthenticatingState {
            static let borderColor = secondaryColor
            static let textColor = primaryColor
            static let backgroundColor = UIColor.white.withAlphaComponent(0.4)
        }
    }

    enum TextField {
        static let placeholderTextColor = UIColor(red: 0.16, green: 0.30, blue: 0.45, alpha: 0.40)
        static let textColor = UIColor(red: 0.16, green: 0.30, blue: 0.45, alpha: 1.0)
        static let backgroundColor = UIColor.white
        static let invalidInputTextColor = UIColor.dangerColor
    }

    enum AppButton {
        static let normalTitleColor = UIColor.white
        static let highlightedTitleColor = UIColor.lightGray
        static let disabledTitleColor = UIColor.lightGray
    }

    enum Switch {
        static let borderColor = UIColor(white: 1.0, alpha: 0.8)
        static let onThumbColor = successColor
        static let offThumbColor = dangerColor
    }

    // Relay availability indicator view
    enum RelayStatusIndicator {
        static let activeColor = successColor.withAlphaComponent(0.9)
        static let inactiveColor = dangerColor.withAlphaComponent(0.95)
        static let highlightColor = UIColor.white
    }

    enum MainSplitView {
        static let dividerColor = UIColor.black
    }

    // Navigation bars
    enum NavigationBar {
        static let backgroundColor = UIColor.secondaryColor
        static let backButtonIndicatorColor = UIColor(white: 1.0, alpha: 0.4)
        static let backButtonTitleColor = UIColor(white: 1.0, alpha: 0.6)
        static let titleColor = UIColor.white
    }

    // Cells
    enum Cell {
        static let backgroundColor = primaryColor
        static let disabledBackgroundColor = backgroundColor.darkened(by: 0.3)!

        static let selectedBackgroundColor = successColor
        static let disabledSelectedBackgroundColor = selectedBackgroundColor.darkened(by: 0.3)!

        static let selectedAltBackgroundColor = backgroundColor.darkened(by: 0.2)!

        static let titleTextColor = UIColor.white
        static let detailTextColor = UIColor(white: 1.0, alpha: 0.8)

        static let disclosureIndicatorColor = UIColor(white: 1.0, alpha: 0.8)
    }

    enum SubCell {
        static let backgroundColor = UIColor(red: 0.15, green: 0.23, blue: 0.33, alpha: 1.0)
    }

    enum SubSubCell {
        static let backgroundColor = UIColor(red: 0.13, green: 0.20, blue: 0.30, alpha: 1.0)
    }

    enum HeaderBar {
        static let defaultBackgroundColor = primaryColor
        static let unsecuredBackgroundColor = dangerColor
        static let securedBackgroundColor = successColor
        static let dividerColor = secondaryColor
        static let brandNameColor = UIColor(white: 1.0, alpha: 0.8)
        static let buttonColor = UIColor(white: 1.0, alpha: 0.8)
        static let disabledButtonColor = UIColor(white: 1.0, alpha: 0.5)
    }

    enum InAppNotificationBanner {
        static let errorIndicatorColor = dangerColor
        static let successIndicatorColor = successColor
        static let warningIndicatorColor = warningColor

        static let titleColor = UIColor.white
        static let bodyColor = UIColor(white: 1.0, alpha: 0.6)
    }

    // Common colors
    static let primaryColor = UIColor(red: 0.16, green: 0.30, blue: 0.45, alpha: 1.0)
    static let secondaryColor = UIColor(red: 0.10, green: 0.18, blue: 0.27, alpha: 1.0)
    static let dangerColor = UIColor(red: 0.89, green: 0.25, blue: 0.22, alpha: 1.0)
    static let warningColor = UIColor(red: 1.0, green: 0.84, blue: 0.14, alpha: 1.0)
    static let successColor = UIColor(red: 0.27, green: 0.68, blue: 0.30, alpha: 1.0)
}
