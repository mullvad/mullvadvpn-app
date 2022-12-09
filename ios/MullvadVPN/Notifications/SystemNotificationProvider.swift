//
//  SystemNotificationProvider.swift
//  MullvadVPN
//
//  Created by pronebird on 09/12/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UserNotifications

/// Protocol describing a system notification provider.
protocol SystemNotificationProvider: NotificationProviderProtocol {
    /// Notification request if available, otherwise `nil`.
    var notificationRequest: UNNotificationRequest? { get }

    /// Whether any pending requests should be removed.
    var shouldRemovePendingRequests: Bool { get }

    /// Whether any delivered requests should be removed.
    var shouldRemoveDeliveredRequests: Bool { get }

    /// Handle system notification response.
    /// Return `true` if response was handled by this provider, otherwise `false`.
    func handleResponse(_ response: UNNotificationResponse) -> Bool
}
