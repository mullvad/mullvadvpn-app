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

/// Enum type describing kinds of in-app actions.
enum InAppNotificationActionType {
    /// Action type represented by an image that depicts a cross tilted by 45 degrees.
    case close

    /// Returns an image associated with the corresponding action type.
    var image: UIImage? {
        return UIImage(named: "IconCloseSml")?.withRenderingMode(.automatic)
    }
}

/// Type describing a specific in-app notification action.
struct InAppNotificationAction: Equatable {
    /// Type of action.
    var type: InAppNotificationActionType

    /// Block handler.
    var handler: (() -> Void)?

    static func == (lhs: InAppNotificationAction, rhs: InAppNotificationAction) -> Bool {
        return lhs.type == rhs.type
    }
}

enum NotificationBannerStyle {
    case success, warning, error
}
