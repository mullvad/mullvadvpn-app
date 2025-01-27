//
//  NotificationProviderIdentifier.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-05-10.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
enum NotificationPriority: Int, Comparable {
    case low = 1
    case medium = 2
    case high = 3
    case critical = 4

    static func < (lhs: NotificationPriority, rhs: NotificationPriority) -> Bool {
        return lhs.rawValue < rhs.rawValue
    }
}

enum NotificationProviderIdentifier: String {
    case accountExpirySystemNotification = "AccountExpiryNotification"
    case accountExpiryInAppNotification = "AccountExpiryInAppNotification"
    case registeredDeviceInAppNotification = "RegisteredDeviceInAppNotification"
    case tunnelStatusNotificationProvider = "TunnelStatusNotificationProvider"
    case latestChangesInAppNotificationProvider = "LatestChangesInAppNotificationProvider"
    case `default` = "default"

    var domainIdentifier: String {
        "net.mullvad.MullvadVPN.\(rawValue)"
    }
}
