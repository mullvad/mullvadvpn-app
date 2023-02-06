//
//  UIPresentationController+Private.swift
//  MullvadVPN
//
//  Created by pronebird on 31/01/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

extension UIPresentationController {
    static let presentationTransitionWillBegin = Notification.Name(
        "UIPresentationControllerPresentationTransitionWillBeginNotification"
    )

    static let presentationTransitionDidEndNotification = Notification.Name(
        "UIPresentationControllerPresentationTransitionDidEndNotification"
    )

    static let dismissalTransitionWillBeginNotification = Notification.Name(
        "UIPresentationControllerDismissalTransitionWillBeginNotification"
    )

    static let dismissalTransitionDidEndNotification = Notification.Name(
        "UIPresentationControllerDismissalTransitionDidEndNotification"
    )

    /// Included in `presentationTransitionDidEndNotification` notifications.
    static let presentationTransitionDidEndCompletedUserInfoKey =
        "UIPresentationControllerPresentationTransitionDidEndCompletedKey"

    /// Included in `dismissalTransitionDidEndNotification` notifications.
    static let dismissalTransitionDidEndCompletedUserInfoKey =
        "UIPresentationControllerDismissalTransitionDidEndCompletedKey"
}
