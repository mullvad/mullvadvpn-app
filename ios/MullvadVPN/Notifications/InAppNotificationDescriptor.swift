//
//  InAppNotificationDescriptor.swift
//  MullvadVPN
//
//  Created by pronebird on 09/12/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Struct describing in-app notification.
struct InAppNotificationDescriptor: Equatable {
    /// Notification identifier.
    var identifier: String

    /// Notification banner style.
    var style: NotificationBannerStyle

    /// Notification title.
    var title: String

    /// Notification body.
    var body: String
}

enum NotificationBannerStyle {
    case success, warning, error
}
