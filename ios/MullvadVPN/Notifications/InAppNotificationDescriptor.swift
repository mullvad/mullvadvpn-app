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
struct InAppNotificationDescriptor {
    /// Notification identifier.
    var identifier: String

    /// Notification banner style.
    var style: NotificationBannerStyle

    /// Notification title.
    var title: String

    /// Notification body.
    var body: NSAttributedString

    /// Notification action
    var action: InAppNotificationAction?
}

extension InAppNotificationDescriptor: Equatable {
    static func == (lhs: InAppNotificationDescriptor, rhs: InAppNotificationDescriptor) -> Bool {
        lhs.identifier == rhs.identifier
    }
}

struct InAppNotificationAction {
    var image: UIImage?
    var handler: (() -> Void)?
}

enum NotificationBannerStyle {
    case success, warning, error
}
