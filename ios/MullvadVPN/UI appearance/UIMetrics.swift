//
//  UIMetrics.swift
//  MullvadVPN
//
//  Created by pronebird on 10/03/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes
import UIKit

enum UIMetrics {
    enum TableView {
        /// Height for separators between cells and/or sections.
        static let separatorHeight: CGFloat = 0.33
        /// Spacing used between distinct sections of views
        static let sectionSpacing: CGFloat = 24
        /// Common layout margins for row views presentation
        /// Similar to `SettingsCell.layoutMargins` however maintains equal horizontal spacing
        static let rowViewLayoutMargins = NSDirectionalEdgeInsets(top: 16, leading: 24, bottom: 16, trailing: 24)
        /// Common cell indentation width
        static let cellIndentationWidth: CGFloat = 16
    }

    enum CustomAlert {
        /// Layout margins for container (main view) in `CustomAlertViewController`
        static let containerMargins = NSDirectionalEdgeInsets(
            top: 28,
            leading: 16,
            bottom: 16,
            trailing: 16
        )

        /// Spacing between view containers in `CustomAlertViewController`
        static let containerSpacing: CGFloat = 16

        /// Spacing between view containers in `CustomAlertViewController`
        static let interContainerSpacing: CGFloat = 4
    }

    enum DimmingView {
        static let opacity: CGFloat = 0.5
        static let cornerRadius: CGFloat = 8
        static let backgroundColor: UIColor = .black
    }

    enum FormSheetTransition {
        static let duration: Duration = .milliseconds(500)
        static let delay: Duration = .zero
        static let animationOptions: UIView.AnimationOptions = [.curveEaseInOut]
    }

    enum SettingsRedeemVoucher {
        static let cornerRadius = 8.0
        static let preferredContentSize = CGSize(width: 280, height: 260)
        static let contentLayoutMargins = NSDirectionalEdgeInsets(top: 16, leading: 16, bottom: 16, trailing: 16)
        static let successfulRedeemMargins = NSDirectionalEdgeInsets(top: 16, leading: 8, bottom: 16, trailing: 8)
    }

    enum AccountDeletion {
        static let preferredContentSize = CGSize(width: 480, height: 640)
    }

    enum Button {
        static let barButtonSize: CGFloat = 44.0
    }

    enum SettingsCell {
        static let textFieldContentInsets = UIEdgeInsets(top: 8, left: 24, bottom: 8, right: 24)
        static let textFieldNonEditingContentInsetLeft: CGFloat = 40
        static let layoutMargins = NSDirectionalEdgeInsets(top: 16, leading: 24, bottom: 16, trailing: 12)
        static let inputCellTextFieldLayoutMargins = UIEdgeInsets(top: 0, left: 8, bottom: 0, right: 8)
        static let selectableSettingsCellLeftViewSpacing: CGFloat = 12
        static let checkableSettingsCellLeftViewSpacing: CGFloat = 20
    }

    enum InAppBannerNotification {
        /// Layout margins for contents presented within the banner.
        static let layoutMargins = NSDirectionalEdgeInsets(top: 16, leading: 24, bottom: 16, trailing: 24)

        /// Size of little round severity indicator.
        static let indicatorSize = CGSize(width: 12, height: 12)
    }

    enum DisconnectSplitButton {
        static let secondaryButtonPhone = CGSize(width: 42, height: 42)
        static let secondaryButtonPad = CGSize(width: 52, height: 52)
    }

    enum FilterView {
        static let labelSpacing: CGFloat = 5
        static let interChipViewSpacing: CGFloat = 8
        static let chipViewCornerRadius: CGFloat = 8
        static let chipViewLayoutMargins = UIEdgeInsets(top: 3, left: 8, bottom: 3, right: 8)
        static let chipViewLabelSpacing: CGFloat = 7
    }

    enum ConnectionPanelView {
        static let inRowHeight: CGFloat = 22
        static let outRowHeight: CGFloat = 44
    }
}

extension UIMetrics {
    /// Spacing used in stack views of buttons
    static let interButtonSpacing: CGFloat = 16

    /// Text field margins
    static let textFieldMargins = UIEdgeInsets(top: 12, left: 14, bottom: 12, right: 14)

    /// Corner radius used for controls such as buttons and text fields
    static let controlCornerRadius: CGFloat = 4

    /// Maximum width of the split view content container on iPad
    static let maximumSplitViewContentContainerWidth: CGFloat = 810 * 0.7

    /// Minimum sidebar width in points
    static let minimumSplitViewSidebarWidth: CGFloat = 300

    /// Maximum sidebar width in percentage points (0...1)
    static let maximumSplitViewSidebarWidthFraction: CGFloat = 0.3

    /// Spacing between buttons in header bar.
    static let headerBarButtonSpacing: CGFloat = 20

    /// Size of a square logo image in header bar.
    static let headerBarLogoSize: CGFloat = 44

    /// Height of brand name. Width is automatically produced based on aspect ratio.
    static let headerBarBrandNameHeight: CGFloat = 18

    /// Various paddings used throughout the app to visually separate elements in StackViews
    static let padding4: CGFloat = 4
    static let padding8: CGFloat = 8
    static let padding10: CGFloat = 10
    static let padding16: CGFloat = 16
    static let padding24: CGFloat = 24
    static let padding32: CGFloat = 32
    static let padding40: CGFloat = 40
    static let padding48: CGFloat = 48

    /// Preferred content size for controllers presented using formsheet modal presentation style.
    static let preferredFormSheetContentSize = CGSize(width: 480, height: 640)

    /// Common layout margins for content presentation
    static let contentLayoutMargins = NSDirectionalEdgeInsets(top: 24, leading: 24, bottom: 24, trailing: 24)

    /// Common content margins for content presentation
    static let contentInsets = UIEdgeInsets(top: 24, left: 24, bottom: 24, right: 24)

    /// Common layout margins for location cell presentation
    static let selectLocationCellLayoutMargins = NSDirectionalEdgeInsets(top: 16, leading: 28, bottom: 16, trailing: 12)
}
