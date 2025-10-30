//
//  UIMetrics.swift
//  MullvadVPN
//
//  Created by pronebird on 10/03/2021.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes
import SwiftUI
import UIKit

enum UIMetrics {
    enum TableView {
        /// Height of a cell.
        static let rowHeight: CGFloat = 56
        /// Height for separators between cells and/or sections.
        static let separatorHeight: CGFloat = 0.33
        /// Spacing used between distinct sections of views
        static let sectionSpacing: CGFloat = 16
        /// Spacing used for empty header views
        static let emptyHeaderHeight: CGFloat = 8
        /// Common cell indentation width
        static let cellIndentationWidth: CGFloat = 16
        /// Spacing for info button
        static let infoButtonSpacing: CGFloat = 8
        /// Heading margins
        static let headingLayoutMargins = NSDirectionalEdgeInsets(top: 0, leading: 0, bottom: 16, trailing: 0)
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

    enum AccessMethodActionSheetTransition {
        static let duration: Duration = .milliseconds(250)
        static let animationOptions: UIView.AnimationOptions = [.curveEaseInOut]
    }

    enum SettingsRedeemVoucher {
        static let cornerRadius: CGFloat = 8
        static let preferredContentSize = CGSize(width: 280, height: 260)
        static let contentLayoutMargins = NSDirectionalEdgeInsets(top: 16, leading: 16, bottom: 16, trailing: 16)
        static let successfulRedeemMargins = NSDirectionalEdgeInsets(top: 16, leading: 8, bottom: 16, trailing: 8)
    }

    enum AccountDeletion {
        static let preferredContentSize = CGSize(width: 480, height: 640)
    }

    enum Button {
        static let barButtonSize: CGFloat = 32
        static let accountInfoSize: CGFloat = 18
        static let minimumTappableAreaSize = CGSize(width: 44, height: 44)
    }

    enum SettingsCell {
        static let textFieldContentInsets = UIEdgeInsets(top: 16, left: 24, bottom: 16, right: 24)
        static let textFieldNonEditingContentInsetLeft: CGFloat = 40
        static let inputCellTextFieldLayoutMargins = UIEdgeInsets(top: 0, left: 8, bottom: 0, right: 8)
        static let selectableSettingsCellLeftViewSpacing: CGFloat = 12
        static let checkableSettingsCellLeftViewSpacing: CGFloat = 12

        /// Cell layout margins used in table views.
        static let defaultLayoutMargins = NSDirectionalEdgeInsets(top: 16, leading: 16, bottom: 16, trailing: 16)

        static let settingsValidationErrorLayoutMargins = NSDirectionalEdgeInsets(
            top: 8,
            leading: 16,
            bottom: 8,
            trailing: 16
        )
        static let apiAccessPickerListContentInsetTop: CGFloat = 16
        static let verticalDividerHeight: CGFloat = 22
        static let detailsButtonSize: CGFloat = 60
        static let infoButtonLeadingMargin: CGFloat = 8
    }

    enum SettingsInfoView {
        static let layoutMargins = EdgeInsets(top: 0, leading: 16, bottom: 0, trailing: 16)
    }

    enum SettingsRowView {
        static let cornerRadius: CGFloat = 10
        static let layoutMargins = EdgeInsets(top: 0, leading: 16, bottom: 0, trailing: 16)
        static let footerLayoutMargins = EdgeInsets(top: 5, leading: 16, bottom: 5, trailing: 16)
    }

    enum InAppBannerNotification {
        /// Layout margins for contents presented within the banner.
        static let layoutMargins = NSDirectionalEdgeInsets(top: 16, leading: 24, bottom: 16, trailing: 24)

        /// Size of little round severity indicator.
        static let indicatorSize = CGSize(width: 12, height: 12)
    }

    enum DisconnectSplitButton {
        static let secondaryButton = CGSize(width: 42, height: 42)
    }

    enum ConnectionPanelView {
        static let inRowHeight: CGFloat = 22
        static let outRowHeight: CGFloat = 44
    }

    enum MainButton {
        static let cornerRadius: CGFloat = 4
    }

    enum FeatureIndicators {
        static let chipViewHorisontalPadding: CGFloat = 8
        static let chipViewTrailingMargin: CGFloat = 6
    }
}

extension UIMetrics {
    /// Spacing used in stack views of buttons
    static let interButtonSpacing: CGFloat = 16

    /// Text field margins
    static let textFieldMargins = UIEdgeInsets(top: 12, left: 14, bottom: 12, right: 14)

    /// Text view margins
    static let textViewMargins = UIEdgeInsets(top: 14, left: 14, bottom: 14, right: 14)

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
    static let contentLayoutMargins = contentInsets.toDirectionalInsets

    /// Common content margins for content presentation
    static let contentInsets = UIEdgeInsets(top: 16, left: 16, bottom: 16, right: 16)

    /// Common layout margins for location cell presentation
    static let locationCellLayoutMargins = NSDirectionalEdgeInsets(top: 16, leading: 16, bottom: 16, trailing: 12)

    /// Padding for the large navigation title
    static let largeNavigationTitlePadding = NSDirectionalEdgeInsets(top: 0, leading: 16, bottom: 0, trailing: 16)

    /// Layout margins used by content heading displayed below the large navigation title.
    static let contentHeadingLayoutMargins = NSDirectionalEdgeInsets(top: 0, leading: 16, bottom: 16, trailing: 16)

    /// Layout margins used by content footer displayed below eg. a list.
    static let contentFooterLayoutMargins = NSDirectionalEdgeInsets(top: 24, leading: 16, bottom: 0, trailing: 16)
}
