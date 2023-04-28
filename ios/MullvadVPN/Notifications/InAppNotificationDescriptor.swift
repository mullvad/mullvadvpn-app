//
//  InAppNotificationDescriptor.swift
//  MullvadVPN
//
//  Created by pronebird on 09/12/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UIKit.UIImage

/// Struct describing in-app notification.
struct InAppNotificationDescriptor: Equatable {
    /// Notification identifier.
    var identifier: String

    /// Notification banner style.
    var style: NotificationBannerStyle

    /// Notification title.
    var title: String

    /// Notification body.
    var body: NSAttributedString

    /// Notification action.
    var action: InAppNotificationAction?
}

/// Type describing a specific in-app notification action.
struct InAppNotificationAction: Equatable {
    /// Image assigned to action button.
    var image: UIImage?

    /// Action handler for button.
    var handler: (() -> Void)?

    static func == (lhs: InAppNotificationAction, rhs: InAppNotificationAction) -> Bool {
        return lhs.image == rhs.image
    }
}

enum NotificationBannerStyle {
    case success, warning, error
}
