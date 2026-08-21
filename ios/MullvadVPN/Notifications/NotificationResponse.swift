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
import UserNotifications

/**
 Struct holding system or in-app notification response.
 */
struct NotificationResponse {
    /// Provider identifier.
    var providerIdentifier: NotificationProviderIdentifier

    /// Action identifier, i.e UNNotificationDefaultActionIdentifier or any custom.
    var actionIdentifier: String

    /// System notification response. Unset for in-app notifications.
    var systemResponse: UNNotificationResponse?
}
