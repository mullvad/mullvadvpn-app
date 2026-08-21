// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation
import MullvadTypes
import UIKit.UIImage

/// Struct describing in-app notification.
struct InAppNotificationDescriptor: Equatable {
    /// Notification identifier.
    var identifier: NotificationProviderIdentifier

    /// Notification banner style.
    var style: NotificationBannerStyle

    /// Notification title.
    var title: String

    /// Notification body.
    var body: NSAttributedString

    /// Notification action.
    var button: InAppNotificationAction?

    /// Notification tap action (optional).
    var tapAction: InAppNotificationAction?
}

/// Type describing a specific in-app notification action.
struct InAppNotificationAction: Equatable {
    /// Image assigned to action button.
    var image: UIImage?

    /// Action handler for button.
    var handler: (() -> Void)?

    static func == (lhs: InAppNotificationAction, rhs: InAppNotificationAction) -> Bool {
        lhs.image == rhs.image
    }
}

enum NotificationBannerStyle {
    case success, warning, error
}
