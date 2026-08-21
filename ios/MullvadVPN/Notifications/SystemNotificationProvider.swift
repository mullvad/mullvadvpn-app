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
import UserNotifications

/// Protocol describing a system notification provider.
protocol SystemNotificationProvider: NotificationProviderProtocol {
    /// Notification request if available, otherwise `nil`.
    var notificationRequest: UNNotificationRequest? { get }

    /// Whether any pending requests should be removed.
    var shouldRemovePendingRequests: Bool { get }

    /// Whether any delivered requests should be removed.
    var shouldRemoveDeliveredRequests: Bool { get }
}
