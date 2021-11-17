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
            static let borderColor = UIColor.clear
            static let textColor = UIColor.white
            static let backgroundColor = UIColor.white.withAlphaComponent(0.2)
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
    }

    enum SubCell {
        static let backgroundColor = namedColor("SubCell")
    }

    enum SubSubCell {
        static let backgroundColor = namedColor("SubSubCell")
    }

    enum HeaderBar {
        static let defaultBackgroundColor = primaryColor
        static let unsecuredBackgroundColor = dangerColor
        static let securedBackgroundColor = successColor
        static let dividerColor = secondaryColor
        static let brandNameColor = UIColor(white: 1.0, alpha: 0.8)
        static let buttonColor = UIColor(white: 1.0, alpha: 0.8)
    }

    enum InAppNotificationBanner {
        static let errorIndicatorColor = dangerColor
        static let successIndicatorColor = successColor
        static let warningIndicatorColor = warningColor
    }

    // Common colors
    static let primaryColor = namedColor("Primary")
    static let secondaryColor = namedColor("Secondary")
    static let dangerColor = namedColor("Danger")
    static let warningColor = namedColor("Warning")
    static let successColor = namedColor("Success")
}

/// This is a helper function to access named colors from the main bundle and circumvent storyboard
/// crash.
/// See: https://openradar.appspot.com/47113341
private func namedColor(_ name: StringLiteralType) -> UIColor {
    UIColor(named: name, in: Bundle(for: AppDelegate.self), compatibleWith: nil)!
}
