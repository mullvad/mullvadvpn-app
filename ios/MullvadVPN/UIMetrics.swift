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
    static var contentLayoutMargins = UIEdgeInsets(top: 24, left: 24, bottom: 24, right: 24)

    /// Layout margins for in-app notification banner presentation
    static var inAppBannerNotificationLayoutMargins = UIEdgeInsets(top: 16, left: 24, bottom: 16, right: 24)

    /// Maximum width of the split view content container on iPad
    static var maximumSplitViewContentContainerWidth: CGFloat = 810 * 0.7

    /// Minimum sidebar width in points
    static var minimumSplitViewSidebarWidth: CGFloat = 300

    /// Maximum sidebar width in percentage points (0...1)
    static var maximumSplitViewSidebarWidthFraction: CGFloat = 0.3

}
