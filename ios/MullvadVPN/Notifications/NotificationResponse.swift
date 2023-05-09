//
//  NotificationResponse.swift
//  MullvadVPN
//
//  Created by pronebird on 09/05/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UserNotifications

/**
 Struct holding system or in-app notification response.
 */
struct NotificationResponse {
    /// Provider identifier.
    var providerIdentifier: String

    /// Action identifier, i.e UNNotificationDefaultActionIdentifier or any custom.
    var actionIdentifier: String

    /// System notification response. Unset for in-app notifications.
    var systemResponse: UNNotificationResponse?
}
