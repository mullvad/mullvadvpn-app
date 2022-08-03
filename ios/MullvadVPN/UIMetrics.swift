//
//  UIMetrics.swift
//  MullvadVPN
//
//  Created by pronebird on 10/03/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import UIKit

enum UIMetrics {}

extension UIMetrics {
    /// Common layout margins for content presentation
    static let contentLayoutMargins = UIEdgeInsets(top: 24, left: 24, bottom: 24, right: 24)

    /// Common layout margins for row views presentation
    /// Similar to `settingsCellLayoutMargins` however maintains equal horizontal spacing
    static let rowViewLayoutMargins = UIEdgeInsets(top: 16, left: 24, bottom: 16, right: 24)

    /// Common layout margins for settings cell presentation
    static let settingsCellLayoutMargins = UIEdgeInsets(top: 16, left: 24, bottom: 16, right: 12)

    /// Common layout margins for location cell presentation
    static let selectLocationCellLayoutMargins = UIEdgeInsets(
        top: 16,
        left: 28,
        bottom: 16,
        right: 12
    )

    /// Common cell indentation width
    static let cellIndentationWidth: CGFloat = 16

    /// Layout margins for in-app notification banner presentation
    static let inAppBannerNotificationLayoutMargins = UIEdgeInsets(
        top: 16,
        left: 24,
        bottom: 16,
        right: 24
    )

    /// Spacing used in stack views of buttons
    static let interButtonSpacing: CGFloat = 16

    /// Spacing used between distinct sections of views
    static let sectionSpacing: CGFloat = 24

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
}
