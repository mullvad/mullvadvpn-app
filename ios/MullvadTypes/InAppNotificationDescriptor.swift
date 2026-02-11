//
//  InAppNotificationDescriptor.swift
//  MullvadVPN
//
//  Created by pronebird on 09/12/2022.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UIKit.UIImage

/// Struct describing in-app notification.
public struct InAppNotificationDescriptor: Equatable {
    /// Notification identifier.
    public var identifier: NotificationProviderIdentifier

    /// Notification banner style.
    public var style: NotificationBannerStyle

    /// Notification title.
    public var title: String

    /// Notification body.
    public var body: NSAttributedString

    /// Notification action (optional).
    public var button: InAppNotificationAction?

    /// Notification button action (optional).
    public var tapAction: InAppNotificationAction?

    public init(
        identifier: NotificationProviderIdentifier,
        style: NotificationBannerStyle,
        title: String,
        body: NSAttributedString,
        button: InAppNotificationAction? = nil,
        tapAction: InAppNotificationAction? = nil
    ) {
        self.identifier = identifier
        self.style = style
        self.title = title
        self.body = body
        self.button = button
        self.tapAction = tapAction
    }
}

/// Type describing a specific in-app notification action.
public struct InAppNotificationAction: Equatable {
    /// Image assigned to action button.
    public var image: UIImage?

    /// Action handler for button.
    public var handler: (() -> Void)?

    public init(image: UIImage? = nil, handler: (() -> Void)?) {
        self.image = image
        self.handler = handler
    }

    public static func == (lhs: InAppNotificationAction, rhs: InAppNotificationAction) -> Bool {
        lhs.image == rhs.image
    }
}

public enum NotificationBannerStyle {
    case success, warning, error
}
